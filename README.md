# aquatic-crawler

![Linux](https://github.com/YGGverse/aquatic-crawler/actions/workflows/linux.yml/badge.svg)
[![Dependencies](https://deps.rs/repo/github/YGGverse/aquatic-crawler/status.svg)](https://deps.rs/repo/github/YGGverse/aquatic-crawler)
[![crates.io](https://img.shields.io/crates/v/aquatic-crawler.svg)](https://crates.io/crates/aquatic-crawler)

Crawler/aggregation tool for the [Aquatic](https://github.com/greatest-ape/aquatic) BitTorrent tracker API.

> [!NOTE]
> Project in development!

## Roadmap

* Targets supported
    * [x] IPv4/IPv6 info-hash JSON/API (see [PR#233](https://github.com/greatest-ape/aquatic/pull/233))
        * [x] local file path
        * [ ] remote URL
* Storage
    * [x] File system (dump as `.torrent`)
        [x] V1
        [ ] V2
    * [ ] [Manticore](https://github.com/manticoresoftware/manticoresearch-rust) full text search
    * [ ] SQLite
* Tools
    * [ ] Storage cleaner
    * [ ] Implement tests

## Install

1. `git clone https://github.com/YGGverse/aquatic-crawler.git && cd aquatic-crawler`
2. `cargo build --release`
3. `sudo install target/release/aquatic-crawler /usr/local/bin/aquatic-crawler`

## Usage

``` bash
aquatic-crawler --infohash-source /path/to/info-hash-ipv4.json\
                --infohash-source /path/to/info-hash-ipv6.json\
                --infohash-source /path/to/another-source.json\
                --torrents-path   /path/to/storage
```
* all arguments are optional, to support multiple source and target drivers
  running without arguments does nothing!

### Options

``` bash
Options:
  -d, --debug <DEBUG>
          Debug level

          * `e` - error * `i` - info

          [default: ei]

  -i, --infohash-source <INFOHASH_SOURCE>
          Filepath(s) to the Aquatic tracker info-hash JSON/API (PR#233)

  -t, --torrents-path <TORRENTS_PATH>
          Directory path to store the `.torrent` files

  -s, --sleep <SLEEP>
          Crawl loop delay in seconds

          [default: 300]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```