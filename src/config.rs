use clap::Parser;
use regex::Regex;
use std::{net::SocketAddr, path::PathBuf};
use url::Url;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Absolute path(s) or URL(s) to import infohashes from the Aquatic tracker binary API
    ///
    /// * PR#233 feature ([Wiki](https://github.com/YGGverse/aquatic-crawler/wiki/Aquatic))
    #[arg(long)]
    pub infohash: Vec<String>,

    /// The P2P Blocklist file URL (to filter outgoing connections)
    ///
    /// * use `--blocklist=file:///path/to/blocklist.txt` format for the local path
    #[arg(long)]
    pub blocklist: Option<Url>,

    /// Define custom tracker(s) to preload the `.torrent` files info
    #[arg(long)]
    pub tracker: Vec<Url>,

    /// Define initial peer(s) to preload the `.torrent` files info
    #[arg(long)]
    pub initial_peer: Option<Vec<SocketAddr>>,

    /// Appends `--tracker` value to magnets and torrents
    #[arg(long, default_value_t = false)]
    pub export_trackers: bool,

    /// Enable DHT resolver
    #[arg(long, default_value_t = false)]
    pub enable_dht: bool,

    /// Enable LSD multicast
    #[arg(long, default_value_t = false)]
    pub enable_lsd: bool,

    /// Disable TCP connection
    #[arg(long, default_value_t = false)]
    pub disable_tcp: bool,

    /// Bind resolver session on specified device name (`tun0`, `mycelium`, etc.)
    #[arg(long)]
    pub bind: Option<String>,

    /// Bind listener on specified `host:port` (`[host]:port` for IPv6)
    ///
    /// * this option is useful only for binding the data exchange service,
    ///   to restrict the outgoing connections for torrent resolver, use `bind` option instead
    #[arg(long)]
    pub listen: Option<SocketAddr>,

    /// Enable UPnP forwarding
    #[arg(long, default_value_t = false)]
    pub listen_upnp: bool,

    /// Enable upload (share bytes received with BitTorrent network)
    #[arg(long, default_value_t = false)]
    pub enable_upload: bool,

    /// Directory path to store preloaded data (e.g. `.torrent` files)
    #[arg(long)]
    pub preload: PathBuf,

    /// Preload only files match regex pattern (list only without preload by default)
    /// * see also `preload_max_filesize`, `preload_max_filecount` options
    ///
    /// ## Example:
    ///
    /// Filter by image ext
    /// ```
    /// --preload-regex '\.(png|gif|jpeg|jpg|webp|svg|log|txt)$'
    /// ```
    ///
    /// * requires `storage` argument defined
    #[arg(long)]
    pub preload_regex: Option<Regex>,

    /// Max size sum of preloaded files per torrent (match `preload_regex`)
    #[arg(long)]
    pub preload_max_filesize: Option<u64>,

    /// Max count of preloaded files per torrent (match `preload_regex`)
    #[arg(long)]
    pub preload_max_filecount: Option<usize>,

    /// Use `socks5://[username:password@]host:port`
    #[arg(long)]
    pub proxy_url: Option<Url>,

    // Peer options
    #[arg(long)]
    pub peer_connect_timeout: Option<u64>,

    #[arg(long)]
    pub peer_read_write_timeout: Option<u64>,

    #[arg(long)]
    pub peer_keep_alive_interval: Option<u64>,

    /// Estimated info-hash index capacity
    #[arg(long, default_value_t = 1000)]
    pub index_capacity: usize,

    /// Max time to handle each torrent
    #[arg(long, default_value_t = 10)]
    pub add_torrent_timeout: u64,

    /// Crawl loop delay in seconds
    #[arg(long, default_value_t = 300)]
    pub sleep: u64,

    /// Limit upload speed (b/s)
    #[arg(long)]
    pub upload_limit: Option<u32>,

    /// Limit download speed (b/s)
    #[arg(long)]
    pub download_limit: Option<u32>,

    /// Skip long-thinking connections,
    /// try to handle the other hashes in this queue after `n` seconds
    #[arg(long, default_value_t = 10)]
    pub wait_until_completed: u64,
}
