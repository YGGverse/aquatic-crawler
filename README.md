# aquatic-crawler

![Build](https://github.com/YGGverse/aquatic-crawler/actions/workflows/build.yml/badge.svg)
[![Dependencies](https://deps.rs/repo/github/YGGverse/aquatic-crawler/status.svg)](https://deps.rs/repo/github/YGGverse/aquatic-crawler)
[![crates.io](https://img.shields.io/crates/v/aquatic-crawler.svg)](https://crates.io/crates/aquatic-crawler)

SSD-friendly FS crawler for the [Aquatic](https://github.com/greatest-ape/aquatic) BitTorrent tracker, based on the [librqbit](https://github.com/ikatson/rqbit/tree/main/crates/librqbit) API

> [!NOTE]
> * requires PR#233, the [yggverse](https://github.com/YGGverse/aquatic/tree/yggverse) or [info-hash-api](https://github.com/YGGverse/aquatic/tree/info-hash-api) branch implementation
> * compatible with any other `--infohash` source in `hash1hash2...` binary format
> * see also the [βtracker](https://github.com/YGGverse/btracker/) as the web frontend for `preload` storage
> * visit project [Wiki](https://github.com/YGGverse/aquatic-crawler/wiki) for details

## Conception

```
torrent client > aquatic_udp > infohash.bin < aquatic-crawler > * /preload/info-hash.torrent
torrent client               <-----------------------|          * /preload/info-hash/data
               <-------------------------------------|          * /preload/.info-hash/tmp
```

## Install

> [!TIP]
> You may want to install some [system dependencies](https://github.com/YGGverse/aquatic-crawler/wiki/Dependencies)

1. `git clone https://github.com/YGGverse/aquatic-crawler.git && cd aquatic-crawler`
2. `cargo build --release`
3. `sudo install target/release/aquatic-crawler /usr/local/bin/aquatic-crawler`

## Usage

> [!TIP]
> * prepend `RUST_LOG=debug` to debug, append `NO_COLOR=1` to disable fmt
> * use `--preload-*` arguments group to avoid preloading everything (by default)
> * make sure the current `nofile` value corresponds to the expected count of torrent files ([details](https://github.com/YGGverse/aquatic-crawler/wiki/Troubleshooting))

``` bash
aquatic-crawler --infohash /path/to/info-hash-ipv4.bin\
                --infohash /path/to/info-hash-ipv6.bin\
                --infohash /path/to/another-source.bin\
                --tracker  udp://host1:port\
                --tracker  udp://host2:port\
                --preload  /path/to/directory\
                --preload-max-filesize=50000\
                --preload-max-filecount=10\
                --preload-regex="\.(png|gif|jpeg|jpg|webp|svg|log|txt)$"
```

### Options

``` bash
aquatic-crawler --help
```
