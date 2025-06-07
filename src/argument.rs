use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Argument {
    /// Debug level
    ///
    /// * `e` - error
    /// * `i` - info
    #[arg(short, long, default_value_t = String::from("ei"))]
    pub debug: String,

    /// Clear previous index collected on crawl session start
    #[arg(short, long, default_value_t = false)]
    pub clear: bool,

    /// Filepath(s) to the Aquatic tracker info-hash JSON/API
    ///
    /// * PR#233 feature
    #[arg(short, long)]
    pub infohash_source: Vec<String>,

    /// Directory path to store the `.torrent` files
    #[arg(long)]
    pub torrents_path: Option<String>,

    /// Define custom tracker(s) to preload the `.torrent` files info
    #[arg(long)]
    pub torrent_tracker: Vec<String>,

    /// Disable DHT resolver (useful with `torrent_tracker`)
    #[arg(long, default_value_t = false)]
    pub disable_dht: bool,

    /// Enable UPnP
    #[arg(long, default_value_t = false)]
    pub enable_upnp_port_forwarding: bool,

    /// Enable upload
    #[arg(long, default_value_t = false)]
    pub enable_upload: bool,

    /// Use `socks5://[username:password@]host:port`
    #[arg(long)]
    pub socks_proxy_url: Option<String>,

    /// Crawl loop delay in seconds
    #[arg(short, default_value_t = 300)]
    pub sleep: u64,
}
