![Logo](logo/logo.png)

# Bitcoin Push Notification Service (BPNS)

## Description

Bitcoin Push Notification Service (BPNS) allows you to receive notifications of Bitcoin transactions of your non-custodial wallets on a provider of your choice, all while respecting your privacy!

Supported notification providers and platforms:

- [Matrix](https://matrix.org) (both notifications and bot)
- [Telegram](https://telegram.org) (both notifications and bot - coming soon)
- [ntfy](https://ntfy.sh/) (only notifications - coming soon)
- [Gotify](https://gotify.net/) (only notifications - coming soon)

## Requirements

- [Bitcoin Core (22.0+)](https://github.com/bitcoin/bitcoin)

## Usage

* [Build from source](doc/build.md) 
* [Configuration](doc/configuration.md) 
* [Usage](doc/usage.md) 

## Features

* Watch both for incoming and outcoming txs of wallets
* Support address derivation from public keys (both single and multi sig)
* Low CPU & memory usage
* Store all data **locally** using a single [RocksDB](https://github.com/rust-rocksdb/rust-rocksdb) database, for better consistency and crash recovery

## State

**This project is in an ALPHA state**

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details
