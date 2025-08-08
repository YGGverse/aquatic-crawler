mod api;
mod config;
mod preload;

use anyhow::Result;
use config::Config;
use librqbit::{
    AddTorrent, AddTorrentOptions, AddTorrentResponse, ConnectionOptions, ListenerOptions,
    PeerConnectionOptions, SessionOptions,
};
use preload::Preload;
use std::{collections::HashSet, num::NonZero, time::Duration};
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    use chrono::Local;
    use clap::Parser;
    use tokio::time;
    // init debug
    if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::fmt::init()
    } // librqbit
    // init components
    let time_init = Local::now();
    let config = Config::parse();
    let preload = Preload::init(
        config.preload,
        config.preload_regex,
        config.preload_max_filecount,
        config.preload_max_filesize,
    )?;
    let session = librqbit::Session::new_with_opts(
        preload.root().clone(),
        SessionOptions {
            bind_device_name: config.bind,
            listen: match config.listen {
                Some(listen_addr) => Some(ListenerOptions {
                    enable_upnp_port_forwarding: config.listen_upnp,
                    listen_addr,
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
                enable_tcp: !config.disable_tcp,
                proxy_url: config.proxy_url.map(|u| u.to_string()),
                peer_opts: Some(PeerConnectionOptions {
                    connect_timeout: config.peer_connect_timeout.map(Duration::from_secs),
                    read_write_timeout: config.peer_read_write_timeout.map(Duration::from_secs),
                    keep_alive_interval: config.peer_keep_alive_interval.map(Duration::from_secs),
                }),
            }),
            disable_dht_persistence: true,
            disable_dht: !config.enable_dht,
            disable_local_service_discovery: !config.enable_lsd,
            disable_upload: !config.enable_upload,
            persistence: None,
            ratelimits: librqbit::limits::LimitsConfig {
                upload_bps: config.upload_limit.and_then(NonZero::new),
                download_bps: config.download_limit.and_then(NonZero::new),
            },
            trackers: config.tracker.iter().cloned().collect(),
            ..SessionOptions::default()
        },
    )
    .await?;
    log::info!("crawler started at {time_init}");
    loop {
        let time_queue = Local::now();
        log::debug!("queue crawl begin at {time_queue}...");
        for source in &config.infohash {
            log::debug!("index source `{source}`...");
            // grab latest info-hashes from this source
            // * aquatic server may update the stats at this moment, handle result manually
            for i in match api::get(source, config.index_capacity) {
                Some(i) => i,
                None => {
                    // skip without panic
                    log::warn!(
                        "the feed `{source}` has an incomplete format (or is still updating); skip."
                    );
                    continue;
                }
            } {
                // convert to string once
                let i = i.as_string();
                if preload.contains_torrent(&i)? {
                    continue;
                }
                log::debug!("index `{i}`...");
                // run the crawler in single thread for performance reasons,
                // use `timeout` argument option to skip the dead connections.
                match time::timeout(
                    Duration::from_secs(config.add_torrent_timeout),
                    session.add_torrent(
                        AddTorrent::from_url(magnet(
                            &i,
                            if config.tracker.is_empty() {
                                None
                            } else {
                                Some(config.tracker.as_ref())
                            },
                        )),
                        Some(AddTorrentOptions {
                            paused: true, // continue after `only_files` init
                            overwrite: true,
                            disable_trackers: config.tracker.is_empty(),
                            initial_peers: config.initial_peer.clone(),
                            list_only: false,
                            // it is important to blacklist all files preload until initiation
                            only_files: Some(Vec::with_capacity(
                                config.preload_max_filecount.unwrap_or_default(),
                            )),
                            // the destination folder to preload files match `preload_regex`
                            // * e.g. images for audio albums
                            output_folder: preload.tmp(&i, true)?.to_str().map(|s| s.to_string()),
                            ..Default::default()
                        }),
                    ),
                )
                .await
                {
                    Ok(r) => match r {
                        // on `preload_regex` case only
                        Ok(AddTorrentResponse::Added(id, mt)) => {
                            let mut keep_files = HashSet::with_capacity(
                                config.preload_max_filecount.unwrap_or_default(),
                            );
                            let mut only_files = HashSet::with_capacity(
                                config.preload_max_filecount.unwrap_or_default(),
                            );
                            mt.wait_until_initialized().await?;
                            let bytes = mt.with_metadata(|m| {
                                for (id, info) in m.file_infos.iter().enumerate() {
                                    if preload
                                        .max_filecount
                                        .is_some_and(|limit| only_files.len() + 1 > limit)
                                    {
                                        log::debug!(
                                            "file count limit ({}) reached, skip file `{id}` for `{i}` at `{}` (and other files after it)",
                                            only_files.len(),
                                            info.relative_filename.to_string_lossy()
                                        );
                                        break;
                                    }
                                    if preload.max_filesize.is_some_and(|limit| info.len > limit) {
                                        log::debug!(
                                            "file size ({}) limit reached, skip file `{id}` for `{i}` at `{}`",
                                            info.len,
                                            info.relative_filename.to_string_lossy()
                                        );
                                        continue;
                                    }
                                    if preload.regex.as_ref().is_some_and(|r| {
                                        !r.is_match(&info.relative_filename.to_string_lossy())
                                    }) {
                                        log::debug!("regex filter, skip file `{id}` for `{i}` at `{}`",
                                        info.relative_filename.to_string_lossy());
                                        continue;
                                    }
                                    log::debug!(
                                        "keep file `{id}` for `{i}` as `{}`",
                                        info.relative_filename.to_string_lossy()
                                    );
                                    assert!(keep_files.insert(info.relative_filename.clone()));
                                    assert!(only_files.insert(id))
                                }
                                m.torrent_bytes.to_vec()
                            })?;
                            session.update_only_files(&mt, &only_files).await?;
                            session.unpause(&mt).await?;
                            log::debug!("begin torrent `{i}` preload...");
                            // await for `preload_regex` files download to continue
                            mt.wait_until_completed().await?;
                            log::debug!("begin torrent `{i}` preload completed.");
                            // persist torrent bytes and preloaded content,
                            // cleanup tmp (see rqbit#408)
                            log::debug!("persist data for torrent `{i}`...");
                            preload.commit(&i, bytes, Some(keep_files))?;
                            // remove torrent from session as indexed
                            session
                                .delete(librqbit::api::TorrentIdOrHash::Id(id), false)
                                .await?;
                            log::debug!("torrent `{i}` resolved.")
                        }
                        Ok(_) => panic!(),
                        Err(e) => log::debug!("failed to resolve `{i}`: `{e}`."),
                    },
                    Err(e) => log::debug!("failed to resolve `{i}`: `{e}`"),
                }
            }
        }
        log::debug!(
            "queue completed at {time_queue} (time: {} / uptime: {}) await {} seconds to continue...",
            Local::now()
                .signed_duration_since(time_queue)
                .as_seconds_f32(),
            Local::now()
                .signed_duration_since(time_init)
                .as_seconds_f32(),
            config.sleep,
        );
        std::thread::sleep(Duration::from_secs(config.sleep))
    }
}

/// Build magnet URI
fn magnet(info_hash: &str, trackers: Option<&Vec<Url>>) -> String {
    let mut m = format!("magnet:?xt=urn:btih:{info_hash}");
    if let Some(t) = trackers {
        for tracker in t {
            m.push_str("&tr=");
            m.push_str(&urlencoding::encode(tracker.as_str()))
        }
    }
    m
}
