// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::time::Instant;

use crate::{
    common::thread,
    core::db::{Mempool, Notification},
    core::STORE,
    util,
};

#[derive(Clone)]
pub struct Cleaner;

impl Cleaner {
    pub fn run() {
        thread::spawn("notification_cleaner", {
            log::info!("Database Notification Cleaner started");
            move || loop {
                let start = Instant::now();
                match Self::notification_cleaner() {
                    Ok(_) => {
                        let elapsed_time = start.elapsed().as_millis();
                        log::info!("Notification cleaned in {} ms", elapsed_time);
                        thread::sleep(3600);
                    }
                    Err(error) => {
                        log::error!("Notification cleaner: {:?} - retrying in 60 sec", error);
                        thread::sleep(60);
                    }
                };
            }
        });

        thread::spawn("mempool_txs_cache_cleaner", {
            log::info!("Mempool Txs Cache Cleaner started");
            move || loop {
                let start = Instant::now();
                match Self::mempool_txs_cache_cleaner() {
                    Ok(_) => {
                        let elapsed_time = start.elapsed().as_millis();
                        log::info!("Mempool txs cache cleaned in {} ms", elapsed_time);
                        thread::sleep(3600);
                    }
                    Err(error) => {
                        log::error!(
                            "Mempool txs cache cleaner: {:?} - retrying in 60 sec",
                            error
                        );
                        thread::sleep(60);
                    }
                };
            }
        });
    }

    fn notification_cleaner() -> Result<(), crate::core::db::Error> {
        let notifications: Vec<Notification> = STORE.get_notifications()?;

        notifications.into_iter().for_each(|notification| {
            let timestamp: u64 = util::timestamp();
            let expire_timestamp: u64 = notification.timestamp + 30 * 24 * 60 * 60; // 30 days

            if timestamp > expire_timestamp {
                match STORE.delete_notifications_by_token_and_ids(
                    notification.token.as_str(),
                    [notification.id].to_vec(),
                ) {
                    Ok(_) => log::debug!("Notification deleted"),
                    Err(error) => {
                        log::warn!("Impossible to delete notification ({:?})", error)
                    }
                };
            }
        });

        Ok(())
    }

    fn mempool_txs_cache_cleaner() -> Result<(), crate::core::db::Error> {
        let mempool_cache: HashMap<String, Mempool> = STORE.get_mempool_txs_cached()?;

        mempool_cache.into_iter().for_each(|(key, value)| {
            let timestamp: u64 = util::timestamp();
            let expire_timestamp: u64 = value.timestamp + 24 * 60 * 60; // 1 day

            if timestamp > expire_timestamp {
                match STORE.remove_mempool_tx_cached(key.as_str()) {
                    Ok(_) => log::trace!("Mempool tx {} deleted", key),
                    Err(error) => {
                        log::warn!("Impossible to delete tx from mempool cache ({:?})", error)
                    }
                };
            }
        });

        Ok(())
    }
}

impl Drop for Cleaner {
    fn drop(&mut self) {
        if thread::panicking() {
            std::process::exit(0x1);
        }
    }
}
