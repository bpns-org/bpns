// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use bitcoin_rpc::{BlockchainInfo, IndexInfo, NetworkInfo, PeerInfo, RpcClient, RpcError};

pub mod address;
mod processor;

use processor::Processor;

use crate::{common::thread, CONFIG};

lazy_static! {
    pub(crate) static ref RPC: RpcClient = RpcClient::new(
        &CONFIG.bitcoin.rpc_addr,
        CONFIG.bitcoin.rpc_username.as_str(),
        CONFIG.bitcoin.rpc_password.as_str(),
    );
}

pub struct Bitcoin;

impl Bitcoin {
    pub fn run() {
        thread::spawn("bitcoin", {
            move || {
                loop {
                    let blockchain_info: BlockchainInfo = match RPC.get_blockchain_info() {
                        Ok(data) => data,
                        Err(error) => {
                            log::error!("Get blockchain info: {:?} - retrying in 60 sec", error);
                            thread::sleep(60);
                            continue;
                        }
                    };
                    let network_info: NetworkInfo = match RPC.get_network_info() {
                        Ok(data) => data,
                        Err(error) => {
                            log::error!("Get network info: {:?} - retrying in 60 sec", error);
                            thread::sleep(60);
                            continue;
                        }
                    };
                    let peers_info: Vec<PeerInfo> = match RPC.get_peer_info() {
                        Ok(data) => data,
                        Err(error) => {
                            log::error!("Get peer info: {:?} - retrying in 60 sec", error);
                            thread::sleep(60);
                            continue;
                        }
                    };
                    let index_info: IndexInfo = match RPC.get_index_info() {
                        Ok(data) => data,
                        Err(error) => match error {
                            RpcError::BadResult => {
                                log::error!("Get index info: {:?} - retrying in 60 sec", error);
                                thread::sleep(60);
                                continue;
                            }
                            _ => {
                                log::error!("You must enable txindex to use this application. Plese add txindex=1 to bitcoin.conf file.");
                                panic!("txindex not enabled");
                            }
                        },
                    };

                    let left_blocks: u32 = blockchain_info.headers - blockchain_info.blocks;

                    if blockchain_info.chain != "main" {
                        log::error!("Invalid blockchain provided. This application require Bitcoin mainnet! Please switch from {} to main net or provide a different rpc.", blockchain_info.chain);
                        panic!("invalid blockchain network");
                    }

                    if blockchain_info.pruned {
                        log::error!("Prune mode detected! This application requires a full node, please disable the prune mode or provide a different rpc.");
                        panic!("prune mode enabled");
                    }

                    if network_info.version < 220000 {
                        log::error!("This application requires Bitcoin Core 22.0+");
                        panic!("Bitcoin Core version incompatible");
                    }

                    if !network_info.network_active {
                        log::error!("This application requires active Bitcoin P2P network.");
                        panic!("P2P network not enabled");
                    }

                    if peers_info.is_empty() {
                        log::info!("Waiting to connect to peers");
                        thread::sleep(10);
                        continue;
                    }

                    if blockchain_info.headers - blockchain_info.blocks == 0 {
                        if !index_info.txindex.synced {
                            log::info!(
                                "Waiting to index {} blocks",
                                blockchain_info.headers - index_info.txindex.best_block_height
                            );
                            thread::sleep(10);
                            continue;
                        }

                        break;
                    }

                    log::info!(
                        "Waiting to download {} blocks{}",
                        left_blocks,
                        if blockchain_info.initial_block_download {
                            " (IBD)"
                        } else {
                            ""
                        }
                    );
                    thread::sleep(10);
                }

                Processor::run();
            }
        });
    }
}

impl Drop for Bitcoin {
    fn drop(&mut self) {
        if thread::panicking() {
            std::process::exit(0x1);
        }
    }
}
