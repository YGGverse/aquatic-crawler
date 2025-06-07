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

    /// Filepath(s) to the Aquatic tracker info-hash JSON/API
    ///
    /// * PR #233 info-hash table implementation has multiple source tables for IPv4 and IPv6
    #[arg(short, long)]
    pub infohash_source: Vec<String>,

    /// Directory path to store the `.torrent` files
    #[arg(short, long)]
    pub torrents_path: Option<String>,

    /// Crawl loop delay in seconds
    #[arg(short, long, default_value_t = 300)]
    pub sleep: u64,
}
