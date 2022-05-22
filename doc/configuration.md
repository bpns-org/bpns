# CONFIGURATION

## Config file

Copy `config-example.toml` file, rename to `config.toml`, edit with your settings and then move to `~/.bpns/config.toml` for Linux and MacOS or `C:\Users\YOUR_USERNAME\.bpns\config.toml` for Windows.

## Bitcoin

Edit your [bitcoin.conf](https://github.com/bitcoin/bitcoin/blob/master/share/examples/bitcoin.conf) file (usually located to `~/.bitcoin/bitcoin.conf`):

- Add `server=1`, `txindex=1` and `purne=0`
- Add auth settings: `rpcuser=USER` and `rpcpassword=PASSWORD` (change USER and PASSWORD)