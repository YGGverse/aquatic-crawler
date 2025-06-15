use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Argument {
    /// Debug level
    ///
    /// * `e` - error
    /// * `i` - info
    /// * `t` - trace (run with `RUST_LOG=librqbit=trace`)
    #[arg(short, long, default_value_t = String::from("ei"))]
    pub debug: String,

    /// Clear previous index collected on crawl session start
    #[arg(long, default_value_t = false)]
    pub clear: bool,

    /// Absolute filename(s) to the Aquatic tracker info-hash JSON/API
    ///
    /// * PR#233 feature
    #[arg(long)]
    pub infohash_file: Vec<String>,

    /// Directory path to store preloaded data (e.g. `.torrent` files)
    #[arg(long)]
    pub storage: String,

    /// Define custom tracker(s) to preload the `.torrent` files info
    #[arg(long)]
    pub torrent_tracker: Vec<String>,

    /// Define initial peer(s) to preload the `.torrent` files info
    #[arg(long)]
    pub initial_peer: Vec<String>,

    /// Enable DHT resolver
    #[arg(long, default_value_t = false)]
    pub enable_dht: bool,

    /// Enable TCP connection
    #[arg(long, default_value_t = false)]
    pub enable_tcp: bool,

    /// Enable UPnP
    #[arg(long, default_value_t = false)]
    pub enable_upnp_port_forwarding: bool,

    /// Enable upload
    #[arg(long, default_value_t = false)]
    pub enable_upload: bool,

    /// Preload files match regex pattern (list only without preload by default)
    ///
    /// ## Example:
    ///
    /// Filter by image ext
    /// ```
    /// --preload-regex '(png|gif|jpeg|jpg|webp)$'
    /// ```
    ///
    /// * requires `storage` argument defined
    #[arg(long)]
    pub preload_regex: Option<String>,

    /// Save resolved torrent files to the `storage` location
    #[arg(long, default_value_t = true)]
    pub save_torrents: bool,

    /// Use `socks5://[username:password@]host:port`
    #[arg(long)]
    pub proxy_url: Option<String>,

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

    /// Max time to download each torrent
    #[arg(long, default_value_t = 10)]
    pub download_torrent_timeout: u64,

    /// Crawl loop delay in seconds
    #[arg(long, default_value_t = 300)]
    pub sleep: u64,

    /// Limit upload speed (b/s)
    #[arg(long)]
    pub upload_limit: Option<u32>,

    /// Limit download speed (b/s)
    #[arg(long)]
    pub download_limit: Option<u32>,
}
