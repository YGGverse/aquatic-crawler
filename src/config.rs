use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Debug level
    ///
    /// * `e` - error
    /// * `i` - info
    /// * `t` - trace (run with `RUST_LOG=librqbit=trace`)
    #[arg(short, long, default_value_t = String::from("ei"))]
    pub debug: String,

    /// Absolute path(s) or URL(s) to import infohashes from the Aquatic tracker JSON/API
    ///
    /// * PR#233 feature
    #[arg(long)]
    pub infohash: Vec<String>,

    /// Define custom tracker(s) to preload the `.torrent` files info
    #[arg(long)]
    pub tracker: Vec<String>,

    /// Define initial peer(s) to preload the `.torrent` files info
    #[arg(long)]
    pub initial_peer: Vec<String>,

    /// Save resolved torrent files to given directory
    #[arg(long)]
    pub export_torrents: Option<String>,

    /// File path to export RSS feed
    #[arg(long)]
    pub export_rss: Option<String>,

    /// Custom title for RSS feed (channel)
    #[arg(long, default_value_t = String::from("aquatic-crawler"))]
    pub export_rss_title: String,

    /// Custom link for RSS feed (channel)
    #[arg(long)]
    pub export_rss_link: Option<String>,

    /// Custom description for RSS feed (channel)
    #[arg(long)]
    pub export_rss_description: Option<String>,

    /// Enable DHT resolver
    #[arg(long, default_value_t = false)]
    pub enable_dht: bool,

    /// Enable TCP connection
    #[arg(long, default_value_t = false)]
    pub enable_tcp: bool,

    /// Enable UPnP
    #[arg(long, default_value_t = false)]
    pub enable_upnp_port_forwarding: bool,

    /// Enable upload (share received bytes with BitTorrent network)
    #[arg(long, default_value_t = false)]
    pub enable_upload: bool,

    /// Directory path to store preloaded data (e.g. `.torrent` files)
    #[arg(long)]
    pub preload: Option<String>,

    /// Clear previous data collected on crawl session start
    #[arg(long, default_value_t = false)]
    pub preload_clear: bool,

    /// Preload only files match regex pattern (list only without preload by default)
    /// * see also `preload_max_filesize`, `preload_max_filecount` options
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

    /// Stop crawler on total preload files size reached
    #[arg(long)]
    pub preload_total_size: Option<u64>,

    /// Max size sum of preloaded files per torrent (match `preload_regex`)
    #[arg(long)]
    pub preload_max_filesize: Option<u64>,

    /// Max count of preloaded files per torrent (match `preload_regex`)
    #[arg(long)]
    pub preload_max_filecount: Option<usize>,

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
