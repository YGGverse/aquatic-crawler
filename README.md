# aquatic-crawler

![Build](https://github.com/YGGverse/aquatic-crawler/actions/workflows/build.yml/badge.svg)
[![Dependencies](https://deps.rs/repo/github/YGGverse/aquatic-crawler/status.svg)](https://deps.rs/repo/github/YGGverse/aquatic-crawler)
[![crates.io](https://img.shields.io/crates/v/aquatic-crawler.svg)](https://crates.io/crates/aquatic-crawler)

SSD-friendly crawler for the [Aquatic](https://github.com/greatest-ape/aquatic) BitTorrent tracker, based on the [librqbit](https://github.com/ikatson/rqbit/tree/main/crates/librqbit) API

> [!NOTE]
> * requires [PR#233](https://github.com/greatest-ape/aquatic/pull/233), see the [Wiki](https://github.com/YGGverse/aquatic-crawler/wiki/Aquatic) for more details
> * compatible with any other `--infohash` source in `hash1hash2...` binary format (see also the [Online API](https://github.com/YGGverse/aquatic-crawler/wiki/Online-API))

## Install

> [!NOTE]
> You may want to install some [system dependencies](https://github.com/YGGverse/aquatic-crawler/wiki/Dependencies)

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
                --preload  /path/to/directory
```
* append `RUST_LOG=debug` to debug

### Options

``` bash
aquatic-crawler --help
```
