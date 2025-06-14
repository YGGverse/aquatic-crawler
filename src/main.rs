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
    use librqbit::{AddTorrent, AddTorrentOptions, AddTorrentResponse, SessionOptions};
    use std::{num::NonZero, time::Duration};

    // init components
    let arg = argument::Argument::parse();
    let debug = Debug::init(&arg.debug)?;
    let peers = peers::Peers::init(&arg.initial_peer)?;
    let storage = Storage::init(&arg.storage, arg.clear)?;
    let trackers = trackers::Trackers::init(&arg.torrent_tracker)?;
    let session = librqbit::Session::new_with_opts(
        storage.path(),
        SessionOptions {
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

    loop {
        debug.info("Index queue begin...");
        let mut total = 0;
        // collect info-hashes from each API channel
        for source in &arg.infohash_file {
            debug.info(&format!("Handle info-hash source `{source}`..."));
            // aquatic server may update the stats at this moment,
            // handle this state manually
            match api::infohashes(source) {
                Ok(infohashes) => {
                    total += infohashes.len();
                    for i in infohashes {
                        debug.info(&format!("Index `{i}`..."));
                        match session
                            .add_torrent(
                                AddTorrent::from_url(format!("magnet:?xt=urn:btih:{i}")),
                                Some(AddTorrentOptions {
                                    overwrite: true,
                                    disable_trackers: trackers.is_empty(),
                                    initial_peers: if peers.is_empty() {
                                        None
                                    } else {
                                        Some(peers.clone())
                                    },
                                    // preload nothing, but listing when regex pattern argument is given
                                    list_only: arg.preload_regex.is_none(),
                                    // this option allows rqbit manager to preload some or any files match pattern
                                    // * useful to build index with multimedia files, like images for audio albums
                                    output_folder: storage.output_folder(&i).ok(),
                                    // applies preload some files to the destination directory (above)
                                    only_files_regex: arg.preload_regex.clone(),
                                    ..Default::default()
                                }),
                            )
                            .await
                        {
                            Ok(r) => match r {
                                AddTorrentResponse::AlreadyManaged(_, t)
                                | AddTorrentResponse::Added(_, t) => {
                                    if arg.save_torrents {
                                        t.with_metadata(|m| {
                                            save_torrent_file(
                                                &storage,
                                                &debug,
                                                &i,
                                                &m.torrent_bytes,
                                            )
                                        })?;
                                    }
                                    /*tokio::spawn({
                                        let t = t.clone();
                                        let d = Duration::from_secs(5);
                                        async move {
                                            loop {
                                                let s = t.stats();
                                                if s.finished {
                                                    break;
                                                }
                                                debug.info(&format!("{s}..."));
                                                tokio::time::sleep(d).await;
                                            }
                                        }
                                    });*/
                                    // @TODO t.wait_until_completed().await?;
                                }
                                AddTorrentResponse::ListOnly(r) => {
                                    if arg.save_torrents {
                                        save_torrent_file(&storage, &debug, &i, &r.torrent_bytes)
                                    }
                                    // @TODO
                                    // use `r.info` for Memory, SQLite, Manticore and other alternative storage type
                                }
                            },
                            Err(e) => debug.info(&format!("Torrent handle skipped: `{e}`")),
                        }
                    }
                }
                Err(e) => debug.error(&e.to_string()),
            }
        }
        debug.info(&format!(
            "Index of {total} hashes completed, await {} seconds to continue...",
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
