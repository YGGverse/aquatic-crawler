mod api;
mod argument;
mod debug;
mod peers;
mod storage;
mod trackers;

use anyhow::Result;
use debug::Debug;
use storage::Storage;

#[tokio::main]
async fn main() -> Result<()> {
    use clap::Parser;
    use librqbit::{
        AddTorrent, AddTorrentOptions, AddTorrentResponse, ConnectionOptions,
        PeerConnectionOptions, SessionOptions,
    };
    use std::{collections::HashSet, num::NonZero, time::Duration};
    use tokio::time;

    // init components
    let arg = argument::Argument::parse();
    let debug = Debug::init(&arg.debug)?;
    let peers = peers::Peers::init(&arg.initial_peer)?;
    let storage = Storage::init(&arg.storage, arg.clear)?;
    let trackers = trackers::Trackers::init(&arg.torrent_tracker)?;
    let session = librqbit::Session::new_with_opts(
        storage.path(),
        SessionOptions {
            connect: Some(ConnectionOptions {
                enable_tcp: arg.enable_tcp,
                proxy_url: arg.proxy_url,
                peer_opts: Some(PeerConnectionOptions {
                    connect_timeout: arg.peer_connect_timeout.map(Duration::from_secs),
                    read_write_timeout: arg.peer_read_write_timeout.map(Duration::from_secs),
                    keep_alive_interval: arg.peer_keep_alive_interval.map(Duration::from_secs),
                }),
            }),
            disable_upload: !arg.enable_upload,
            disable_dht: !arg.enable_dht,
            disable_dht_persistence: true,
            persistence: None,
            ratelimits: librqbit::limits::LimitsConfig {
                upload_bps: arg.upload_limit.and_then(NonZero::new),
                download_bps: arg.download_limit.and_then(NonZero::new),
            },
            trackers: trackers.clone(),
            ..SessionOptions::default()
        },
    )
    .await?;

    // begin
    debug.info("Crawler started");

    // collect processed info hashes to skip on the next iterations (for this session)
    let mut index = HashSet::with_capacity(arg.index_capacity);
    loop {
        debug.info("Index queue begin...");
        for source in &arg.infohash_file {
            debug.info(&format!("Index source `{source}`..."));
            // grab latest info-hashes from this source
            // * aquatic server may update the stats at this moment, handle result manually
            match api::infohashes(source) {
                Ok(infohashes) => {
                    for i in infohashes {
                        // is already indexed?
                        if index.contains(&i) {
                            continue;
                        }
                        debug.info(&format!("Index `{i}`..."));
                        // run the crawler in single thread for performance reasons,
                        // use `timeout` argument option to skip the dead connections.
                        match time::timeout(
                            Duration::from_secs(arg.add_torrent_timeout),
                            session.add_torrent(
                                AddTorrent::from_url(format!("magnet:?xt=urn:btih:{i}")),
                                Some(AddTorrentOptions {
                                    overwrite: true,
                                    disable_trackers: trackers.is_empty(),
                                    initial_peers: peers.initial_peers(),
                                    list_only: arg.preload_regex.is_none(),
                                    // the destination folder to preload files match `only_files_regex`
                                    // * e.g. images for audio albums
                                    output_folder: storage.output_folder(&i, true).ok(),
                                    only_files_regex: arg.preload_regex.clone(),
                                    ..Default::default()
                                }),
                            ),
                        )
                        .await
                        {
                            Ok(r) => match r {
                                // on `preload_regex` case only
                                Ok(AddTorrentResponse::Added(id, mt)) => {
                                    if arg.save_torrents {
                                        mt.with_metadata(|m| {
                                            save_torrent_file(
                                                &storage,
                                                &debug,
                                                &i,
                                                &m.torrent_bytes,
                                            )
                                            // @TODO
                                            // use `r.info` for Memory, SQLite, Manticore and other alternative storage type
                                        })?;
                                    }
                                    // await for `preload_regex` files download to continue
                                    match time::timeout(
                                        Duration::from_secs(arg.download_torrent_timeout),
                                        mt.wait_until_completed(),
                                    )
                                    .await
                                    {
                                        Ok(r) => {
                                            if let Err(e) = r {
                                                debug.info(&format!("Skip `{i}`: `{e}`."))
                                            } else {
                                                // remove torrent from session as indexed
                                                session
                                                    .delete(
                                                        librqbit::api::TorrentIdOrHash::Id(id),
                                                        false,
                                                    )
                                                    .await?;
                                                // cleanup irrelevant files (see rqbit#408)
                                                if let Some(ref r) = arg.preload_regex {
                                                    storage.cleanup(&i, r)?;
                                                }
                                                // ignore on the next crawl iterations for this session
                                                index.insert(i);
                                            }
                                        }
                                        Err(e) => debug.info(&format!("Skip `{i}`: `{e}`.")),
                                    }
                                }
                                Ok(AddTorrentResponse::ListOnly(r)) => {
                                    if arg.save_torrents {
                                        save_torrent_file(&storage, &debug, &i, &r.torrent_bytes)
                                    }
                                    // @TODO
                                    // use `r.info` for Memory, SQLite,
                                    // Manticore and other alternative storage type

                                    // ignore on the next crawl iterations for this session
                                    index.insert(i);
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
        debug.info(&format!(
            "Index completed, {} total, await {} seconds to continue...",
            index.len(),
            arg.sleep,
        ));
        std::thread::sleep(Duration::from_secs(arg.sleep));
    }
}

fn save_torrent_file(s: &Storage, d: &Debug, i: &str, b: &[u8]) {
    if s.torrent_exists(i) {
        d.info(&format!("Torrent file `{i}` already exists, skip"))
    } else {
        match s.save_torrent(i, b) {
            Ok(r) => d.info(&format!("Add torrent file `{}`", r.to_string_lossy())),
            Err(e) => d.error(&e.to_string()),
        }
    }
}
