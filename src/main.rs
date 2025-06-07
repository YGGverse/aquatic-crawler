mod api;
mod argument;
mod database;
mod debug;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use clap::Parser;
    use librqbit::SessionOptions;
    use std::str::FromStr;

    let argument = argument::Argument::parse();

    // calculate debug level once
    let is_debug_i = argument.debug.contains("i");
    let is_debug_e = argument.debug.contains("e");

    // init shared members
    let torrent_storage = if let Some(t) = argument.storage {
        let s = database::torrent::Storage::init(&t, argument.clear)?;
        if argument.clear && is_debug_i {
            debug::info(String::from("Cleanup torrent storage"));
        }
        Some(s)
    } else {
        None
    };

    let mut trackers = std::collections::HashSet::with_capacity(argument.torrent_tracker.len());
    for tracker in argument.torrent_tracker {
        trackers.insert(url::Url::from_str(&tracker)?);
    }

    let mut peers = Vec::with_capacity(argument.initial_peer.len());
    for peer in argument.initial_peer {
        peers.push(std::net::SocketAddr::from_str(&peer)?);
    }

    // begin
    if is_debug_i {
        debug::info(String::from("Crawler started"));
    }

    loop {
        if is_debug_i {
            debug::info(String::from("New index session begin..."));
        }
        let mut total = 0;
        let session = librqbit::Session::new_with_opts(
            std::path::PathBuf::new(),
            SessionOptions {
                disable_dht: !argument.enable_dht,
                disable_upload: !argument.enable_upload,
                enable_upnp_port_forwarding: argument.enable_upnp_port_forwarding,
                socks_proxy_url: argument.socks_proxy_url.clone(),
                trackers: trackers.clone(),
                ..SessionOptions::default()
            },
        )
        .await?;
        // collect info-hashes from API
        for source in &argument.infohash_file {
            if is_debug_i {
                debug::info(format!("Handle info-hash source `{source}`..."));
            }

            // aquatic server may update the stats at this moment,
            // handle this state manually
            match api::infohashes(source) {
                Ok(infohashes) => {
                    total += infohashes.len();
                    for i in infohashes {
                        if torrent_storage.as_ref().is_some_and(|s| !s.exists(&i)) {
                            if is_debug_i {
                                debug::info(format!("Resolve `{i}`..."));
                            }
                            match session
                                .add_torrent(
                                    librqbit::AddTorrent::from_url(format!(
                                        "magnet:?xt=urn:btih:{i}"
                                    )),
                                    Some(librqbit::AddTorrentOptions {
                                        disable_trackers: trackers.is_empty(),
                                        initial_peers: if peers.is_empty() {
                                            None
                                        } else {
                                            Some(peers.clone())
                                        },
                                        list_only: true,
                                        ..Default::default()
                                    }),
                                )
                                .await?
                            {
                                librqbit::AddTorrentResponse::ListOnly(r) => {
                                    if let Some(ref s) = torrent_storage {
                                        let p = s.save(&i, &r.torrent_bytes)?;
                                        if is_debug_i {
                                            debug::info(format!(
                                                "Add new torrent file `{}`",
                                                p.to_string_lossy()
                                            ));
                                        }
                                    }
                                    // @TODO
                                    // use `r.info` for Memory, SQLite, Manticore and other alternative storage type
                                }
                                _ => panic!(),
                            }
                        }
                    }
                }
                Err(ref e) => {
                    if is_debug_e {
                        debug::error(e)
                    }
                }
            }
        }
        session.stop().await;
        if is_debug_i {
            debug::info(format!(
                "Index of {total} hashes completed, await {} seconds to continue...",
                argument.sleep,
            ));
        }
        std::thread::sleep(std::time::Duration::from_secs(argument.sleep));
    }
}
