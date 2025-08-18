mod api;
mod config;
mod preload;

use anyhow::Result;
use config::Config;
use librqbit::{
    AddTorrent, AddTorrentOptions, AddTorrentResponse, ConnectionOptions, SessionOptions,
};
use preload::Preload;
use std::{collections::HashSet, num::NonZero, time::Duration};
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    use chrono::Local;
    use clap::Parser;
    use tokio::time;
    // debug
    if std::env::var("RUST_LOG").is_ok() {
        use tracing_subscriber::fmt::*;
        struct T;
        impl time::FormatTime for T {
            fn format_time(&self, w: &mut format::Writer<'_>) -> std::fmt::Result {
                write!(w, "{}", Local::now())
            }
        }
        fmt().with_timer(T).init()
    }
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
            blocklist_url: config.blocklist.map(|b| b.into()),
            listen: None,
            connect: Some(ConnectionOptions {
                enable_tcp: !config.disable_tcp,
                proxy_url: config.proxy_url.map(|u| u.to_string()),
                ..ConnectionOptions::default()
            }),
            disable_dht_persistence: true,
            disable_dht: !config.enable_dht,
            disable_local_service_discovery: !config.enable_lsd,
            disable_upload: true,
            persistence: None,
            ratelimits: librqbit::limits::LimitsConfig {
                download_bps: config.download_limit.and_then(NonZero::new),
                ..librqbit::limits::LimitsConfig::default()
            },
            trackers: config.tracker.iter().cloned().collect(),
            ..SessionOptions::default()
        },
    )
    .await?;
    let mut ban = HashSet::with_capacity(config.index_capacity);
    log::info!("crawler started at {time_init}");
    loop {
        let time_queue = Local::now();
        log::debug!("queue crawl begin at {time_queue}...");

        // build unique ID index from the multiple info-hash sources
        let mut queue = HashSet::with_capacity(config.index_capacity);
        for source in &config.infohash {
            log::debug!("index source `{source}`...");
            // grab latest info-hashes from this source
            // * aquatic server may update the stats at this moment, handle result manually
            for i in match api::get(source, config.index_capacity) {
                Some(i) => {
                    log::debug!("fetch `{}` hashes from `{source}`...", i.len());
                    i
                }
                None => {
                    log::warn!(
                        "the feed `{source}` has an incomplete format (or is still updating); skip."
                    );
                    continue; // skip without panic
                }
            } {
                queue.insert(i);
            }
        }

        // clean up nonexistent ban entries from the memory pool
        ban.retain(|i| {
            let is_keep = queue.contains(i);
            if !is_keep {
                log::debug!(
                    "remove `{}` from the ban list, as it is no longer available in the source.",
                    i.as_string()
                )
            }
            is_keep
        });

        // handle
        log::debug!(
            "fetched {} unique hashes from {} source, banned: {}.",
            queue.len(),
            config.infohash.len(),
            ban.len()
        );
        for i in queue {
            // convert to string once
            let h = i.as_string();
            if preload.contains_torrent(&h)? {
                log::debug!("torrent `{h}` exists, skip.");
                continue;
            }
            // skip banned entry, remove it from the ban list to retry on the next iteration
            if ban.remove(&i) {
                log::debug!("torrent `{h}` is banned, skip for this queue.");
                continue;
            }
            log::info!("resolve `{h}`...");
            // run the crawler in single thread for performance reasons,
            // use `timeout` argument option to skip the dead connections.
            match time::timeout(
                Duration::from_secs(config.timeout),
                session.add_torrent(
                    AddTorrent::from_url(magnet(
                        &h,
                        if config.tracker.is_empty() {
                            None
                        } else {
                            Some(config.tracker.as_ref())
                        },
                    )),
                    Some(AddTorrentOptions {
                        paused: true, // continue after `only_files` update
                        overwrite: true,
                        disable_trackers: config.tracker.is_empty(),
                        initial_peers: config.initial_peer.clone(),
                        list_only: false,
                        // the destination folder to preload files match `preload_regex`
                        // * e.g. images for audio albums
                        output_folder: preload.tmp_dir(&h, true)?.to_str().map(|s| s.to_string()),
                        ..Default::default()
                    }),
                ),
            )
            .await
            {
                Ok(r) => match r {
                    Ok(AddTorrentResponse::Added(id, mt)) => {
                        assert!(mt.is_paused());
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
                                            "file count limit ({}) reached, skip file `{id}` for `{h}` at `{}` (and other files after it)",
                                            only_files.len(),
                                            info.relative_filename.to_string_lossy()
                                        );
                                        break;
                                    }
                                    if preload.max_filesize.is_some_and(|limit| info.len > limit) {
                                        log::debug!(
                                            "file size ({}) limit reached, skip file `{id}` for `{h}` at `{}`",
                                            info.len,
                                            info.relative_filename.to_string_lossy()
                                        );
                                        continue;
                                    }
                                    if preload.regex.as_ref().is_some_and(|r| {
                                        !r.is_match(&info.relative_filename.to_string_lossy())
                                    }) {
                                        log::debug!("regex filter, skip file `{id}` for `{h}` at `{}`",
                                        info.relative_filename.to_string_lossy());
                                        continue;
                                    }
                                    log::debug!(
                                        "keep file `{id}` for `{h}` as `{}`",
                                        info.relative_filename.to_string_lossy()
                                    );
                                    assert!(keep_files.insert(info.relative_filename.clone()));
                                    assert!(only_files.insert(id))
                                }
                                m.torrent_bytes.to_vec()
                            })?;
                        session.update_only_files(&mt, &only_files).await?;
                        session.unpause(&mt).await?;
                        log::debug!("begin torrent `{h}` preload...");
                        if let Err(e) = time::timeout(
                            Duration::from_secs(config.timeout),
                            mt.wait_until_completed(),
                        )
                        .await
                        {
                            log::info!(
                                "skip awaiting the completion of preload torrent data for `{h}` (`{e}`), ban for the next queue.",
                            );
                            assert!(ban.insert(i));
                            session
                                .delete(librqbit::api::TorrentIdOrHash::Id(id), false)
                                .await?; // * do not collect billions of slow torrents in the session pool
                            continue;
                        }
                        log::debug!("torrent `{h}` preload completed.");
                        // persist torrent bytes and preloaded content,
                        // cleanup tmp (see rqbit#408)
                        log::debug!("persist torrent `{h}` with `{}` files...", keep_files.len());
                        preload.commit(&h, bytes, Some(keep_files))?;
                        session
                            .delete(librqbit::api::TorrentIdOrHash::Id(id), false)
                            .await?; // torrent data was moved on commit; there's no sense in keeping it
                        log::info!("torrent `{h}` resolved.")
                    }
                    Ok(_) => panic!(),
                    Err(e) => {
                        log::warn!(
                            "failed to resolve torrent `{h}`: `{e}`, ban for the next queue."
                        );
                        assert!(ban.insert(i))
                    }
                },
                Err(e) => {
                    log::info!(
                        "skip awaiting the completion of adding torrent `{h}` (`{e}`), ban for the next queue."
                    );
                    assert!(ban.insert(i))
                }
            }
        }
        log::info!(
            "queue completed at {time_queue} (time: {} / uptime: {} / banned: {}) await {} seconds to continue...",
            Local::now()
                .signed_duration_since(time_queue)
                .as_seconds_f32(),
            Local::now()
                .signed_duration_since(time_init)
                .as_seconds_f32(),
            ban.len(),
            config.sleep
        );
        std::thread::sleep(Duration::from_secs(config.sleep))
    }
}

/// Build magnet URI (`librqbit` impl dependency)
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
