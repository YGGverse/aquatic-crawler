use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Argument {
    /// Debug level
    ///
    /// * `e` - error
    /// * `i` - info
    /// * `t` - trace (e.g. to run with `RUST_LOG=librqbit=trace`)
    #[arg(short, long, default_value_t = String::from("ei"))]
    pub debug: String,

    /// Clear previous index collected on crawl session start
    #[arg(short, long, default_value_t = false)]
    pub clear: bool,

    /// Absolute filename(s) to the Aquatic tracker info-hash JSON/API
    ///
    /// * PR#233 feature
    #[arg(long)]
    pub infohash_file: Vec<String>,

    /// Directory path to store reload data (e.g. `.torrent` files)
    #[arg(long)]
    pub storage: Option<String>,

    /// Define custom tracker(s) to preload the `.torrent` files info
    #[arg(long)]
    pub torrent_tracker: Vec<String>,

    /// Define initial peer(s) to preload the `.torrent` files info
    #[arg(long)]
    pub initial_peer: Vec<String>,

    /// Enable DHT resolver
    #[arg(long, default_value_t = false)]
    pub enable_dht: bool,

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
