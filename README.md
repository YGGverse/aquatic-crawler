# aquatic-crawler

![Build](https://github.com/YGGverse/aquatic-crawler/actions/workflows/build.yml/badge.svg)
[![Dependencies](https://deps.rs/repo/github/YGGverse/aquatic-crawler/status.svg)](https://deps.rs/repo/github/YGGverse/aquatic-crawler)
[![crates.io](https://img.shields.io/crates/v/aquatic-crawler.svg)](https://crates.io/crates/aquatic-crawler)

Crawler for [Aquatic](https://github.com/greatest-ape/aquatic) BitTorrent tracker based on [librqbit](https://github.com/ikatson/rqbit/tree/main/crates/librqbit) API

> [!NOTE]
> This crawler is compatible with any other `--infohash` source in `hash1hash2...` binary format!

## Features

> [!TIP]
> For details on all implemented features, see the [Options](#options) section

* Info-hash versions
    * [x] 1
    * [ ] 2
* Import sources
    * [x] IPv4 / IPv6 info-hash binary API (requires [PR#233](https://github.com/greatest-ape/aquatic/pull/233), [Wiki](https://github.com/YGGverse/aquatic-crawler/wiki/Aquatic))
        * [x] local file path
        * [ ] remote URL
* Export options
    * [x] Content (`--preload`)
        * [x] data match the regex pattern (`--preload-regex`)
        * [x] data match limits (see `--preload-*` options group)
    * [x] Resolved `.torrent` files (`--export-torrents`)
    * [x] RSS feed (`--export-rss`) includes resolved torrent meta and magnet links to download
        * customize feed options with `--export-rss-*` options group
    * [ ] [Manticore](https://github.com/manticoresoftware/manticoresearch-rust) full text search index
    * [ ] SQLite database index

## Install

1. `git clone https://github.com/YGGverse/aquatic-crawler.git && cd aquatic-crawler`
2. `cargo build --release`
3. `sudo install target/release/aquatic-crawler /usr/local/bin/aquatic-crawler`

## Usage

``` bash
aquatic-crawler --infohash /path/to/info-hash-ipv4.bin\
                --infohash /path/to/info-hash-ipv6.bin\
                --infohash /path/to/another-source.bin\
                --tracker  udp://host1:port\
                --tracker  udp://host2:port\
                --preload  /path/to/directory\
                --enable-tcp
```

### Options

``` bash
  -d, --debug <DEBUG>
          Debug level

          * `e` - error
          * `i` - info
          * `t` - trace (run with `RUST_LOG=librqbit=trace`)

          [default: ei]

      --infohash <INFOHASH>
          Absolute path(s) or URL(s) to import infohashes from the Aquatic tracker binary API

          * PR#233 feature

      --tracker <TRACKER>
          Define custom tracker(s) to preload the `.torrent` files info

      --initial-peer <INITIAL_PEER>
          Define initial peer(s) to preload the `.torrent` files info

      --export-torrents <EXPORT_TORRENTS>
          Save resolved torrent files to given directory

      --export-rss <EXPORT_RSS>
          File path to export RSS feed

      --export-rss-title <EXPORT_RSS_TITLE>
          Custom title for RSS feed (channel)

          [default: aquatic-crawler]

      --export-rss-link <EXPORT_RSS_LINK>
          Custom link for RSS feed (channel)

      --export-rss-description <EXPORT_RSS_DESCRIPTION>
          Custom description for RSS feed (channel)

      --export-trackers
          Appends `--tracker` value to magnets and torrents

      --enable-dht
          Enable DHT resolver

      --enable-tcp
          Enable TCP connection

      --enable-upnp-port-forwarding
          Enable UPnP

      --enable-upload
          Enable upload (share bytes received with BitTorrent network)

      --preload <PRELOAD>
          Directory path to store preloaded data (e.g. `.torrent` files)

      --preload-clear
          Clear previous data collected on crawl session start

      --preload-regex <PRELOAD_REGEX>
          Preload only files match regex pattern (list only without preload by default)
          * see also `preload_max_filesize`, `preload_max_filecount` options

          ## Example:

          Filter by image ext ``` --preload-regex '(png|gif|jpeg|jpg|webp)$' ```

          * requires `storage` argument defined

      --preload-total-size <PRELOAD_TOTAL_SIZE>
          Stop crawler on total preload files size reached

      --preload-max-filesize <PRELOAD_MAX_FILESIZE>
          Max size sum of preloaded files per torrent (match `preload_regex`)

      --preload-max-filecount <PRELOAD_MAX_FILECOUNT>
          Max count of preloaded files per torrent (match `preload_regex`)

      --proxy-url <PROXY_URL>
          Use `socks5://[username:password@]host:port`

      --peer-connect-timeout <PEER_CONNECT_TIMEOUT>


      --peer-read-write-timeout <PEER_READ_WRITE_TIMEOUT>


      --peer-keep-alive-interval <PEER_KEEP_ALIVE_INTERVAL>


      --index-capacity <INDEX_CAPACITY>
          Estimated info-hash index capacity

          [default: 1000]

      --index-timeout <INDEX_TIMEOUT>
          Remove records from index older than `seconds`

      --add-torrent-timeout <ADD_TORRENT_TIMEOUT>
          Max time to handle each torrent

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