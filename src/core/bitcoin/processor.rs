// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::convert::From;
use std::time::Instant;

use bitcoin_rpc::{Block, RpcError, Transaction, TxOut};
use rayon::prelude::*;

use crate::{common::thread, core::bitcoin::RPC, core::STORE};

#[derive(Debug)]
pub enum ProcessorError {
    Db(crate::common::db::Error),
    Rpc(RpcError),
}

pub struct Processor;

impl Processor {
    pub fn run() {
        // Number of confirmations required to consider the block confirmed.
        const OFFSET: u8 = 5;

        thread::spawn("block_processor", {
            log::info!("Bitcoin Block Processor started");
            move || loop {
                let block_height: u32 = match RPC.get_block_count() {
                    Ok(data) => {
                        log::debug!("Current block is {}", data);
                        data - OFFSET as u32
                    }
                    Err(error) => {
                        log::error!("Get block height: {:?}", error);
                        thread::sleep(60);
                        continue;
                    }
                };

                let last_processed_block: u32 = match STORE.get_last_processed_block() {
                    Ok(value) => value,
                    Err(_) => {
                        let _ = STORE.set_last_processed_block(block_height);
                        block_height
                    }
                };

                log::debug!("Last processed block is {}", last_processed_block);

                if block_height <= last_processed_block {
                    log::debug!("Wait for new block");
                    thread::sleep(120);
                    continue;
                }

                let next_block_to_process: u32 = last_processed_block + 1;
                let start = Instant::now();
                match Self::process_block(next_block_to_process) {
                    Ok(_) => {
                        let elapsed_time = start.elapsed().as_millis();
                        log::trace!(
                            "Block {} processed in {} ms",
                            next_block_to_process,
                            elapsed_time
                        );
                        let _ = STORE.set_last_processed_block(next_block_to_process);
                    }
                    Err(error) => {
                        log::error!("Process block: {:?} - retrying in 60 sec", error);
                        thread::sleep(60);
                    }
                };
            }
        });

        thread::spawn("mempool_processor", {
            log::info!("Bitcoin Mempool Processor started");
            move || loop {
                let start = Instant::now();
                match Self::process_mempool() {
                    Ok(_) => {
                        let elapsed_time = start.elapsed().as_millis();
                        log::trace!("Mempool processed in {} ms", elapsed_time);
                        thread::sleep(3);
                    }
                    Err(error) => {
                        log::error!("Process mempool: {:?} - retrying in 60 sec", error);
                        thread::sleep(60);
                    }
                };
            }
        });
    }

    fn process_block(block_height: u32) -> Result<(), ProcessorError> {
        let block_hash: String = RPC.get_block_hash(block_height)?;
        let block: Block = RPC.get_block(block_hash.as_str())?;

        log::info!("Processing block {} ({} txs)", block_height, block.tx.len());

        block.tx.into_iter().for_each(|mut tx| {
            let main_txid: &str = tx.txid.as_str();

            tx.vin.iter_mut().for_each(|input| {
                if let Some(input_txid) = &input.txid {
                    if let Some(vout) = input.vout {
                        if let Ok(prev_raw_transaction) =
                            RPC.get_raw_transaction(input_txid.as_str())
                        {
                            prev_raw_transaction.vout.into_iter().for_each(|output| {
                                if let Some(output_n) = output.n {
                                    if output_n == vout {
                                        input.prevout = Some(output);
                                    }
                                }
                            });
                        }
                    }
                }
            });

            Self::process_transaction(main_txid, tx.clone(), true);
            let _ = STORE.remove_mempool_tx_cached(main_txid);

            thread::sleep_millis(100);
        });

        Ok(())
    }

    fn process_mempool() -> Result<(), ProcessorError> {
        let raw_mempool: Vec<String> = RPC.get_raw_mempool()?;

        let new_txs: Vec<String> = raw_mempool
            .into_par_iter()
            .filter(|txid| !STORE.is_mempool_tx_cached(txid.as_str()))
            .collect();

        log::debug!("Processing mempool ({} txs)", new_txs.len());

        // Process mempool and queue notification.
        new_txs.into_iter().for_each(|txid| {
            let main_txid: &str = txid.as_str();
            match RPC.get_raw_transaction_with_prevouts(main_txid) {
                Ok(raw_transaction) => {
                    Self::process_transaction(main_txid, raw_transaction, false);
                    let _ = STORE.set_mempool_tx_cached(main_txid);
                }
                Err(error) => log::warn!(
                    "Impossible to get txid {} (mempool_processor - {:?})",
                    main_txid,
                    error
                ),
            };
        });

        Ok(())
    }

    fn process_transaction(txid: &str, tx: Transaction, is_confirmed: bool) {
        let mut inputs: HashMap<String, TxOut> = HashMap::new();

        // Process tx inputs
        tx.vin.into_iter().for_each(|input| {
            if let Some(prevout) = input.prevout {
                if let Some(address) = prevout.clone().script_pub_key.address {
                    if let Some(x) = inputs.get_mut(&address) {
                        let mut new_prevout = x.clone();
                        new_prevout.value += prevout.value;
                        *x = new_prevout;
                    } else {
                        inputs.insert(address, prevout);
                    }
                }
            }
        });

        // Process tx outputs
        tx.vout.into_iter().for_each(|output| {
            if let Some(address) = output.clone().script_pub_key.address {
                if let Some(input) = inputs.get(&address) {
                    if output.value < input.value {
                        let mut new_output = input.clone();
                        new_output.value -= output.value;
                        Self::queue_notification(txid, new_output, "out", is_confirmed);
                    } else {
                        let mut new_output = output;
                        new_output.value -= input.value;
                        Self::queue_notification(txid, new_output, "in", is_confirmed);
                    }

                    inputs.remove(&address);
                } else {
                    Self::queue_notification(txid, output, "in", is_confirmed);
                }
            }
        });

        inputs.into_values().into_iter().for_each(|input| {
            Self::queue_notification(txid, input, "out", is_confirmed);
        });
    }

    fn queue_notification(txid: &str, tx_out: TxOut, tx_type: &str, confirmed: bool) {
        if let Some(address) = tx_out.script_pub_key.address {
            if let Ok(result) = STORE.get_address(address.as_str()) {
                let tokens = result.tokens;

                let amount: u64 = (tx_out.value * 100_000_000.0) as u64;

                let mut counter: u32 = 0;

                tokens.into_iter().for_each(|token| {
                    let notification = STORE.create_notification(
                        token.as_str(),
                        address.as_str(),
                        txid,
                        tx_type,
                        amount,
                        confirmed,
                    );

                    if notification.is_ok() {
                        counter += 1;
                    }
                });

                if counter > 0 {
                    log::info!(
                        "Queued {} notifications for txid {} ({} - {})",
                        counter,
                        txid,
                        tx_type,
                        if confirmed {
                            "confirmed"
                        } else {
                            "unconfirmed"
                        }
                    );
                }
            }
        }
    }
}

impl Drop for Processor {
    fn drop(&mut self) {
        if thread::panicking() {
            std::process::exit(0x1);
        }
    }
}

impl From<crate::common::db::Error> for ProcessorError {
    fn from(err: crate::common::db::Error) -> Self {
        ProcessorError::Db(err)
    }
}

impl From<RpcError> for ProcessorError {
    fn from(err: RpcError) -> Self {
        ProcessorError::Rpc(err)
    }
}
