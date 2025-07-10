mod api;
mod config;
mod format;
mod index;
mod peers;
mod preload;
mod rss;
mod torrent;
mod trackers;

use anyhow::Result;
use config::Config;
use index::Index;
use librqbit::{
    AddTorrent, AddTorrentOptions, AddTorrentResponse, ByteBufOwned, ConnectionOptions,
    ListenerOptions, PeerConnectionOptions, SessionOptions, TorrentMetaV1Info,
};
use peers::Peers;
use rss::Rss;
use std::{collections::HashSet, num::NonZero, path::PathBuf, str::FromStr, time::Duration};
use torrent::Torrent;
use trackers::Trackers;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    use chrono::Local;
    use clap::Parser;
    use tokio::time;

    // init components
    let time_init = Local::now();
    let config = Config::parse();
    if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::fmt::init()
    }
    let peers = Peers::init(&config.initial_peer)?;
    let preload = preload::init(
        config.preload,
        config.preload_regex,
        config.preload_max_filecount,
        config.preload_max_filesize,
        config.preload_total_size,
        config.preload_clear,
    )?;
    let trackers = Trackers::init(&config.tracker)?;
    let torrent = config.export_torrents.map(|p| Torrent::init(&p).unwrap());
    let session = librqbit::Session::new_with_opts(
        match preload {
            Some(ref p) => p.path(),
            None => PathBuf::new(),
        },
        SessionOptions {
            listen: match config.listen {
                Some(l) => Some(ListenerOptions {
                    listen_addr: std::net::SocketAddr::from_str(&l)?,
                    enable_upnp_port_forwarding: config.listen_upnp,
                    ..ListenerOptions::default()
                }),
                None => {
                    if config.listen_upnp {
                        panic!("`--listen` argument is required!")
                    }
                    None
                }
            },
            connect: Some(ConnectionOptions {
                enable_tcp: config.enable_tcp,
                proxy_url: config.proxy_url,
                peer_opts: Some(PeerConnectionOptions {
                    connect_timeout: config.peer_connect_timeout.map(Duration::from_secs),
                    read_write_timeout: config.peer_read_write_timeout.map(Duration::from_secs),
                    keep_alive_interval: config.peer_keep_alive_interval.map(Duration::from_secs),
                }),
            }),
            disable_upload: !config.enable_upload,
            disable_dht: !config.enable_dht,
            disable_dht_persistence: true,
            persistence: None,
            ratelimits: librqbit::limits::LimitsConfig {
                upload_bps: config.upload_limit.and_then(NonZero::new),
                download_bps: config.download_limit.and_then(NonZero::new),
            },
            trackers: trackers.list().clone(),
            ..SessionOptions::default()
        },
    )
    .await?;

    // begin
    println!("Crawler started on {time_init}");
    let mut index = Index::init(
        config.index_capacity,
        config.index_timeout,
        config.export_rss.is_some(),
        config.export_rss.is_some(),
        config.export_rss.is_some() && config.index_list,
    );
    loop {
        let time_queue = Local::now();
        if config.debug {
            println!("\tQueue crawl begin on {time_queue}...")
        }
        index.refresh();
        for source in &config.infohash {
            if config.debug {
                println!("\tIndex source `{source}`...")
            }
            // grab latest info-hashes from this source
            // * aquatic server may update the stats at this moment, handle result manually
            for i in match api::get(source, config.index_capacity) {
                Some(i) => i,
                None => {
                    // skip without panic
                    if config.debug {
                        eprintln!(
                            "The feed `{source}` has an incomplete format (or is still updating); skip."
                        )
                    }
                    continue;
                }
            } {
                // convert to string once
                let i = i.to_string();
                // is already indexed?
                if index.has(&i) {
                    continue;
                }
                if config.debug {
                    println!("\t\tIndex `{i}`...")
                }
                // run the crawler in single thread for performance reasons,
                // use `timeout` argument option to skip the dead connections.
                match time::timeout(
                    Duration::from_secs(config.add_torrent_timeout),
                    session.add_torrent(
                        AddTorrent::from_url(magnet(
                            &i,
                            if config.export_trackers && !trackers.is_empty() {
                                Some(trackers.list())
                            } else {
                                None
                            },
                        )),
                        Some(AddTorrentOptions {
                            paused: true, // continue after `only_files` init
                            overwrite: true,
                            disable_trackers: trackers.is_empty(),
                            initial_peers: peers.initial_peers(),
                            list_only: preload.as_ref().is_none_or(|p| p.regex.is_none()),
                            // it is important to blacklist all files preload until initiation
                            only_files: Some(Vec::with_capacity(
                                config.preload_max_filecount.unwrap_or_default(),
                            )),
                            // the destination folder to preload files match `only_files_regex`
                            // * e.g. images for audio albums
                            output_folder: preload
                                .as_ref()
                                .map(|p| p.output_folder(&i, true).unwrap()),
                            ..Default::default()
                        }),
                    ),
                )
                .await
                {
                    Ok(r) => match r {
                        // on `preload_regex` case only
                        Ok(AddTorrentResponse::Added(id, mt)) => {
                            let mut only_files_size = 0;
                            let mut only_files_keep = Vec::with_capacity(
                                config.preload_max_filecount.unwrap_or_default(),
                            );
                            let mut only_files = HashSet::with_capacity(
                                config.preload_max_filecount.unwrap_or_default(),
                            );
                            mt.wait_until_initialized().await?;
                            let (name, size, list) = mt.with_metadata(|m| {
                                // init preload files list
                                if let Some(ref p) = preload {
                                    for (id, info) in m.file_infos.iter().enumerate() {
                                        if p.matches(info.relative_filename.to_str().unwrap()) {
                                            if p.max_filesize.is_some_and(|limit| {
                                                only_files_size + info.len > limit
                                            }) {
                                                if config.debug {
                                                    println!(
                                                        "\t\t\ttotal files size limit `{i}` reached!"
                                                    )
                                                }
                                                break;
                                            }
                                            if p.max_filecount
                                                .is_some_and(|limit| only_files.len() + 1 > limit)
                                            {
                                                if config.debug {
                                                    println!(
                                                        "\t\t\ttotal files count limit for `{i}` reached!"
                                                    )
                                                }
                                                break;
                                            }
                                            only_files_size += info.len;
                                            if let Some(ref p) = preload {
                                                only_files_keep
                                                    .push(p.absolute(&i, &info.relative_filename))
                                            }
                                            only_files.insert(id);
                                        }
                                    }
                                }
                                if let Some(ref t) = torrent {
                                    save_torrent_file(t, &i, &m.torrent_bytes, config.debug)
                                }

                                (
                                    m.info.name.as_ref().map(|n| n.to_string()),
                                    size(&m.info),
                                    list(&m.info, config.index_list_limit),
                                )
                            })?;
                            session.update_only_files(&mt, &only_files).await?;
                            session.unpause(&mt).await?;
                            // await for `preload_regex` files download to continue
                            mt.wait_until_completed().await?;
                            // remove torrent from session as indexed
                            session
                                .delete(librqbit::api::TorrentIdOrHash::Id(id), false)
                                .await?;
                            // cleanup irrelevant files (see rqbit#408)
                            if let Some(p) = &preload {
                                p.cleanup(&i, Some(only_files_keep))?
                            }

                            if config.debug {
                                println!("\t\t\tadd `{i}` to index.")
                            }

                            index.insert(i, only_files_size, size, list, name)
                        }
                        Ok(AddTorrentResponse::ListOnly(r)) => {
                            if let Some(ref t) = torrent {
                                save_torrent_file(t, &i, &r.torrent_bytes, config.debug)
                            }

                            // @TODO
                            // use `r.info` for Memory, SQLite,
                            // Manticore and other alternative storage type

                            if config.debug {
                                println!("\t\t\tadd `{i}` to index.")
                            }

                            index.insert(
                                i,
                                0,
                                size(&r.info),
                                list(&r.info, config.index_list_limit),
                                r.info.name.map(|n| n.to_string()),
                            )
                        }
                        // unexpected as should be deleted
                        Ok(AddTorrentResponse::AlreadyManaged(..)) => panic!(),
                        Err(e) => eprintln!("Failed to resolve `{i}`: `{e}`."),
                    },
                    Err(e) => {
                        if config.debug {
                            println!("\t\t\tfailed to resolve `{i}`: `{e}`")
                        }
                    }
                }
            }
        }

        if let Some(ref export_rss) = config.export_rss
            && index.is_changed()
        {
            let mut rss = Rss::new(
                export_rss,
                &config.export_rss_title,
                &config.export_rss_link,
                &config.export_rss_description,
                if config.export_trackers && !trackers.is_empty() {
                    Some(trackers.list().clone())
                } else {
                    None
                },
            )?;
            for (k, v) in index.list() {
                rss.push(
                    k,
                    v.name().unwrap_or(k),
                    rss::item_description(v.size(), v.list()),
                    Some(&v.time.to_rfc2822()),
                )?
            }
            rss.commit()?
        }
        if preload
            .as_ref()
            .is_some_and(|p| p.total_size.is_some_and(|s| index.nodes() > s))
        {
            panic!("Preload content size {} bytes reached!", 0)
        }
        if config.debug {
            println!(
                "Queue completed on {time_queue}\n\ttotal: {}\n\ttime: {} s\n\tuptime: {} s\n\tawait {} seconds to continue...",
                index.len(),
                Local::now()
                    .signed_duration_since(time_queue)
                    .as_seconds_f32(),
                Local::now()
                    .signed_duration_since(time_init)
                    .as_seconds_f32(),
                config.sleep,
            )
        }
        std::thread::sleep(Duration::from_secs(config.sleep))
    }
}

/// Shared handler function to save resolved torrents as file
fn save_torrent_file(t: &Torrent, i: &str, b: &[u8], d: bool) {
    match t.persist(i, b) {
        Ok(r) => {
            if d {
                match r {
                    Some(p) => println!("\t\t\tadd torrent file `{}`", p.to_string_lossy()),
                    None => println!("\t\t\ttorrent file `{i}` already exists"),
                }
            }
        }
        Err(e) => eprintln!("Error on save torrent file `{i}`: `{e}`"),
    }
}

/// Build magnet URI
fn magnet(infohash: &str, trackers: Option<&HashSet<Url>>) -> String {
    let mut m = if infohash.len() == 40 {
        format!("magnet:?xt=urn:btih:{infohash}")
    } else {
        todo!("infohash v2 is not supported by librqbit")
    };
    if let Some(t) = trackers {
        for tracker in t {
            m.push_str("&tr=");
            m.push_str(&urlencoding::encode(tracker.as_str()))
        }
    }
    m
}

/// Count total size, including torrent files
fn size(info: &TorrentMetaV1Info<ByteBufOwned>) -> u64 {
    let mut t = 0;
    if let Some(l) = info.length {
        t += l
    }
    if let Some(ref files) = info.files {
        for f in files {
            t += f.length
        }
    }
    t
}

fn list(
    info: &TorrentMetaV1Info<ByteBufOwned>,
    limit: usize,
) -> Option<Vec<(Option<String>, u64)>> {
    info.files.as_ref().map(|files| {
        let mut b = Vec::with_capacity(files.len());
        let mut i = files.iter();
        let mut t = 0;
        for f in i.by_ref() {
            if t < limit {
                t += 1;
                b.push((
                    String::from_utf8(
                        f.path
                            .iter()
                            .enumerate()
                            .flat_map(|(n, b)| {
                                if n == 0 {
                                    b.0.to_vec()
                                } else {
                                    let mut p = vec![b'/'];
                                    p.extend(b.0.to_vec());
                                    p
                                }
                            })
                            .collect(),
                    )
                    .ok(),
                    f.length,
                ));
                continue;
            }
            // limit reached: count sizes left and use placeholder as the last item name
            let mut l = 0;
            for f in i.by_ref() {
                l += f.length
            }
            b.push((Some("...".to_string()), l));
            break;
        }
        b[..t].sort_by(|a, b| a.0.cmp(&b.0)); // @TODO optional
        b
    })
}
