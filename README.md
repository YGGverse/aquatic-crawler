# aquatic-crawler

![Build](https://github.com/YGGverse/aquatic-crawler/actions/workflows/build.yml/badge.svg)
[![Dependencies](https://deps.rs/repo/github/YGGverse/aquatic-crawler/status.svg)](https://deps.rs/repo/github/YGGverse/aquatic-crawler)
[![crates.io](https://img.shields.io/crates/v/aquatic-crawler.svg)](https://crates.io/crates/aquatic-crawler)

Crawler/aggregation tool for the [Aquatic](https://github.com/greatest-ape/aquatic) BitTorrent tracker API.

> [!NOTE]
> Project in development!

## Roadmap

* Targets supported
    * [x] IPv4/IPv6 info-hash JSON/API (requires [PR#233](https://github.com/greatest-ape/aquatic/pull/233))
        * [x] local file path
        * [ ] remote URL
* Storage
    * [x] File system (dump as `.torrent`)
        * [x] V1
        * [ ] V2
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
                --storage         /path/to/storage
```
* all arguments are optional, to support multiple source and target drivers
* running without arguments does nothing!

### Options

``` bash
-d, --debug <DEBUG>
        Debug level

        * `e` - error * `i` - info

        [default: ei]

-c, --clear
        Clear previous index collected on crawl session start

--infohash-file <INFOHASH_FILE>
        Absolute filename(s) to the Aquatic tracker info-hash JSON/API

        * PR#233 feature

--storage <STORAGE>
        Directory path to store reload data (e.g. `.torrent` files)

--torrent-tracker <TORRENT_TRACKER>
        Define custom tracker(s) to preload the `.torrent` files info

--initial-peer <INITIAL_PEER>
        Define initial peer(s) to preload the `.torrent` files info

--enable-dht
        Enable DHT resolver

--enable-upnp-port-forwarding
        Enable UPnP

--enable-upload
        Enable upload

--socks-proxy-url <SOCKS_PROXY_URL>
        Use `socks5://[username:password@]host:port`

-s <SLEEP>
        Crawl loop delay in seconds

        [default: 300]

-h, --help
        Print help (see a summary with '-h')

-V, --version
        Print version
```