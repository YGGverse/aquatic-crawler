# aquatic-crawler

![Build](https://github.com/YGGverse/aquatic-crawler/actions/workflows/build.yml/badge.svg)
[![Dependencies](https://deps.rs/repo/github/YGGverse/aquatic-crawler/status.svg)](https://deps.rs/repo/github/YGGverse/aquatic-crawler)
[![crates.io](https://img.shields.io/crates/v/aquatic-crawler.svg)](https://crates.io/crates/aquatic-crawler)

Crawler for [Aquatic](https://github.com/greatest-ape/aquatic) BitTorrent tracker based on [librqbit](https://github.com/ikatson/rqbit/tree/main/crates/librqbit) API

> [!NOTE]
> Project in development!

## Roadmap

* Info-hash versions supported
    * [x] 1
    * [ ] 2
* Targets supported
    * [x] IPv4/IPv6 info-hash JSON/API (requires [PR#233](https://github.com/greatest-ape/aquatic/pull/233))
        * [x] local file path
        * [ ] remote URL
* Storage
    * [x] File system (resolve infohash to the `.torrent` + download content files match the regex pattern)
    * [ ] [Manticore](https://github.com/manticoresoftware/manticoresearch-rust) full text search
    * [ ] SQLite

## Install

1. `git clone https://github.com/YGGverse/aquatic-crawler.git && cd aquatic-crawler`
2. `cargo build --release`
3. `sudo install target/release/aquatic-crawler /usr/local/bin/aquatic-crawler`

## Usage

``` bash
aquatic-crawler --infohash-file   /path/to/info-hash-ipv4.json\
                --infohash-file   /path/to/info-hash-ipv6.json\
                --infohash-file   /path/to/another-source.json\
                --torrent-tracker udp://host1:port\
                --torrent-tracker udp://host2:port\
                --storage         /path/to/storage\
                --enable-tcp
```

### Options

``` bash
-d, --debug <DEBUG>
        Debug level

        * `e` - error * `i` - info * `t` - trace (run with `RUST_LOG=librqbit=trace`)

        [default: ei]

--clear
        Clear previous index collected on crawl session start

--infohash-file <INFOHASH_FILE>
        Absolute filename(s) to the Aquatic tracker info-hash JSON/API

        * PR#233 feature

--storage <STORAGE>
        Directory path to store preloaded data (e.g. `.torrent` files)

--torrent-tracker <TORRENT_TRACKER>
        Define custom tracker(s) to preload the `.torrent` files info

--initial-peer <INITIAL_PEER>
        Define initial peer(s) to preload the `.torrent` files info

--enable-dht
        Enable DHT resolver

--enable-tcp
        Enable TCP connection

--enable-upnp-port-forwarding
        Enable UPnP

--enable-upload
        Enable upload

--preload-regex <PRELOAD_REGEX>
        Preload files match regex pattern (list only without preload by default)

        ## Example:

        Filter by image ext ``` --preload-regex '(png|gif|jpeg|jpg|webp)$' ```

        * requires `storage` argument defined

--save-torrents
        Save resolved torrent files to the `storage` location

--proxy-url <PROXY_URL>
        Use `socks5://[username:password@]host:port`

--peer-connect-timeout <PEER_CONNECT_TIMEOUT>


--peer-read-write-timeout <PEER_READ_WRITE_TIMEOUT>


--peer-keep-alive-interval <PEER_KEEP_ALIVE_INTERVAL>


--index-capacity <INDEX_CAPACITY>
        Estimated info-hash index capacity

        [default: 1000]

--add-torrent-timeout <ADD_TORRENT_TIMEOUT>
        Max time to handle each torrent

        [default: 10]

--download-torrent-timeout <DOWNLOAD_TORRENT_TIMEOUT>
        Max time to download each torrent

        [default: 10]

--sleep <SLEEP>
        Crawl loop delay in seconds

        [default: 300]

--upload-limit <UPLOAD_LIMIT>
        Limit upload speed (b/s)

--download-limit <DOWNLOAD_LIMIT>
        Limit download speed (b/s)

-h, --help
        Print help (see a summary with '-h')

-V, --version
        Print version
```