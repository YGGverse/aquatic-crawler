mod api;
mod argument;
mod database;
mod debug;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use clap::Parser;
    let argument = argument::Argument::parse();

    // calculate debug level once
    let is_debug_i = argument.debug.contains("i");
    let is_debug_e = argument.debug.contains("e");

    // init shared members
    let torrent_storage = if let Some(t) = argument.torrents_path {
        Some(database::torrent::Storage::init(&t)?)
    } else {
        None
    };

    if is_debug_i {
        debug::info(String::from("Crawler started"));
    }

    loop {
        if is_debug_i {
            debug::info(String::from("New index session begin..."));
        }
        let mut total = 0;
        let session = librqbit::Session::new(std::path::PathBuf::new()).await?;
        // collect info-hashes from API
        for source in &argument.infohash_source {
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
                            match session
                                .add_torrent(
                                    librqbit::AddTorrent::from_url(format!(
                                        "magnet:?xt=urn:btih:{i}"
                                    )),
                                    Some(librqbit::AddTorrentOptions {
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
