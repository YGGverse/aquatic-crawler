mod api;
mod config;
mod debug;
mod index;
mod peers;
mod rss;
mod storage;
mod torrent;
mod trackers;

use anyhow::Result;
use debug::Debug;
use index::Index;
use rss::Rss;
use std::{collections::HashSet, num::NonZero, time::Duration};
use storage::Storage;
use torrent::Torrent;
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
    let config = config::Config::parse();
    let debug = Debug::init(&config.debug)?;
    let peers = peers::Peers::init(&config.initial_peer)?;
    let storage = Storage::init(&config.storage, config.clear)?;
    let trackers = trackers::Trackers::init(&config.torrent_tracker)?;
    let torrent = config.export_torrents.map(|p| Torrent::init(&p).unwrap());
    let preload_regex = config
        .preload_regex
        .map(|ref r| regex::Regex::new(r).unwrap());
    let session = librqbit::Session::new_with_opts(
        storage.path(),
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
    let mut index = Index::init(config.index_capacity);
    loop {
        debug.info("Index queue begin...");
        index.refresh();
        for source in &config.infohash_file {
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
                                    list_only: preload_regex.is_none(),
                                    // it is important to blacklist all files preload until initiation
                                    only_files: Some(Vec::with_capacity(
                                        config.preload_max_filecount.unwrap_or_default(),
                                    )),
                                    // the destination folder to preload files match `only_files_regex`
                                    // * e.g. images for audio albums
                                    output_folder: storage.output_folder(&i, true).ok(),
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
                                        if let Some(ref regex) = preload_regex {
                                            for (id, info) in m.file_infos.iter().enumerate() {
                                                if regex.is_match(
                                                    info.relative_filename.to_str().unwrap(),
                                                ) {
                                                    if config.preload_max_filesize.is_some_and(
                                                        |limit| only_files_size + info.len > limit,
                                                    ) {
                                                        debug.info(&format!(
                                                            "Total files size limit `{i}` reached!"
                                                        ));
                                                        break;
                                                    }
                                                    if config.preload_max_filecount.is_some_and(
                                                        |limit| only_files.len() + 1 > limit,
                                                    ) {
                                                        debug.info(&format!(
                                                            "Total files count limit for `{i}` reached!"
                                                        ));
                                                        break;
                                                    }
                                                    only_files_size += info.len;
                                                    only_files_keep.push(storage.absolute(&i, &info.relative_filename));
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
                                    storage.cleanup(&i, Some(only_files_keep))?;

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
                    v.name.as_ref().unwrap_or(k),
                    None, // @TODO
                    Some(&v.time.to_rfc2822()),
                )?
            }
            rss.commit()?
        }
        if config.preload_total_size.is_some_and(|s| index.nodes() > s) {
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
        Ok(r) => match r {
            Some(p) => d.info(&format!("Add torrent file `{}`", p.to_string_lossy())),
            None => d.info(&format!("Torrent file `{i}` already exists")),
        },
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
