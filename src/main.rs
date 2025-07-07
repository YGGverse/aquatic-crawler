mod api;
mod config;
mod debug;
mod index;
mod peers;
mod preload;
mod rss;
mod torrent;
mod trackers;

use anyhow::Result;
use config::Config;
use debug::Debug;
use index::Index;
use peers::Peers;
use rss::Rss;
use std::{collections::HashSet, num::NonZero, path::PathBuf, time::Duration};
use torrent::Torrent;
use trackers::Trackers;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    use clap::Parser;
    use librqbit::{
        AddTorrent, AddTorrentOptions, AddTorrentResponse, ConnectionOptions,
        PeerConnectionOptions, SessionOptions,
    };
    use tokio::time;

    // init components
    let config = Config::parse();
    let debug = Debug::init(&config.debug)?;
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
            trackers: trackers.clone(),
            ..SessionOptions::default()
        },
    )
    .await?;

    // begin
    debug.info("Crawler started");
    let mut index = Index::init(config.index_capacity, config.export_rss.is_some());
    loop {
        debug.info("Index queue begin...");
        index.refresh();
        for source in &config.infohash {
            if source.contains("://") {
                todo!("URL sources yet not supported")
            }
            debug.info(&format!("Index source `{source}`..."));
            // grab latest info-hashes from this source
            // * aquatic server may update the stats at this moment, handle result manually
            match api::infohashes(source) {
                Ok(infohashes) => {
                    for i in infohashes {
                        // is already indexed?
                        if index.has(&i) {
                            continue;
                        }
                        debug.info(&format!("Index `{i}`..."));
                        // run the crawler in single thread for performance reasons,
                        // use `timeout` argument option to skip the dead connections.
                        match time::timeout(
                            Duration::from_secs(config.add_torrent_timeout),
                            session.add_torrent(
                                AddTorrent::from_url(magnet(&i, None)),
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
                                    let name = mt.with_metadata(|m| {
                                        // init preload files list
                                        if let Some(ref p) = preload {
                                            for (id, info) in m.file_infos.iter().enumerate() {
                                                if p.matches(info.relative_filename.to_str().unwrap()) {
                                                    if p.max_filesize.is_some_and(
                                                        |limit| only_files_size + info.len > limit,
                                                    ) {
                                                        debug.info(&format!(
                                                            "Total files size limit `{i}` reached!"
                                                        ));
                                                        break;
                                                    }
                                                    if p.max_filecount.is_some_and(
                                                        |limit| only_files.len() + 1 > limit,
                                                    ) {
                                                        debug.info(&format!(
                                                            "Total files count limit for `{i}` reached!"
                                                        ));
                                                        break;
                                                    }
                                                    only_files_size += info.len;
                                                    if let Some(ref p) = preload {
                                                        only_files_keep.push(
                                                            p.absolute(&i, &info.relative_filename))
                                                    }
                                                    only_files.insert(id);
                                                }
                                            }
                                        }
                                        if let Some(ref t) = torrent {
                                            save_torrent_file(t, &debug, &i, &m.torrent_bytes)
                                        }
                                        m.info.name.as_ref().map(|n|n.to_string())
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

                                    index.insert(i, only_files_size, name)
                                }
                                Ok(AddTorrentResponse::ListOnly(r)) => {
                                    if let Some(ref t) = torrent {
                                        save_torrent_file(t, &debug, &i, &r.torrent_bytes)
                                    }

                                    // @TODO
                                    // use `r.info` for Memory, SQLite,
                                    // Manticore and other alternative storage type

                                    index.insert(i, 0, r.info.name.map(|n| n.to_string()))
                                }
                                // unexpected as should be deleted
                                Ok(AddTorrentResponse::AlreadyManaged(..)) => panic!(),
                                Err(e) => debug.info(&format!("Skip `{i}`: `{e}`.")),
                            },
                            Err(e) => debug.info(&format!("Skip `{i}`: `{e}`.")),
                        }
                    }
                }
                Err(e) => debug.error(&format!("API issue for `{source}`: `{e}`")),
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
                Some(trackers.clone()),
            )?;
            for (k, v) in index.list() {
                rss.push(
                    k,
                    v.name().unwrap_or(k),
                    None, // @TODO
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
        debug.info(&format!(
            "Index completed, {} total, await {} seconds to continue...",
            index.len(),
            config.sleep,
        ));
        std::thread::sleep(Duration::from_secs(config.sleep));
    }
}

/// Shared handler function to save resolved torrents as file
fn save_torrent_file(t: &Torrent, d: &Debug, i: &str, b: &[u8]) {
    match t.persist(i, b) {
        Ok(r) => d.info(&match r {
            Some(p) => format!("Add torrent file `{}`", p.to_string_lossy()),
            None => format!("Torrent file `{i}` already exists"),
        }),
        Err(e) => d.error(&format!("Error on save torrent file `{i}`: {e}")),
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
