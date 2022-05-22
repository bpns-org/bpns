// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::path::Path;

pub(crate) mod cleaner;

use crate::{
    common::db::{Error, Store},
    util,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Address {
    pub tokens: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub token: String,
    pub address: String,
    pub txid: String,
    pub txtype: String,
    pub amount: u64,
    pub confirmed: bool,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mempool {
    pub timestamp: u64,
}

pub(crate) struct CoreStore {
    pub db: Store,
}

const NETWORK_CF: &str = "network";
const ADDRESS_CF: &str = "address";
const NOTIFICATION_CF: &str = "notification";
const TOKEN_CF: &str = "token";
const MEMPOOL_CF: &str = "mempool";

const COLUMN_FAMILIES: &[&str] = &[
    NETWORK_CF,
    ADDRESS_CF,
    NOTIFICATION_CF,
    TOKEN_CF,
    MEMPOOL_CF,
];

impl CoreStore {
    pub fn open(path: &Path) -> Result<Self, Error> {
        Ok(Self {
            db: Store::open(path, COLUMN_FAMILIES)?,
        })
    }

    pub fn get_last_processed_block(&self) -> Result<u32, Error> {
        let cf = self.db.cf_handle(NETWORK_CF);
        match self.db.get(cf, "last_processed_block") {
            Ok(result) => match util::convert::bytes_to_number::<u32>(result) {
                Some(num) => Ok(num),
                None => Err(Error::FailedToDeserialize),
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_last_processed_block(&self, block_height: u32) -> Result<(), Error> {
        let cf = self.db.cf_handle(NETWORK_CF);
        self.db
            .put(cf, "last_processed_block", block_height.to_string())
    }

    pub fn create_token(&self, token: &str) -> Result<(), Error> {
        let cf = self.db.cf_handle(TOKEN_CF);

        if !util::is_token(token) {
            return Err(Error::InvalidValue);
        }

        if self.token_exist(token) {
            return Err(Error::AlreadyExist);
        }

        self.db.put::<&str, &str>(cf, token, "")
    }

    pub fn token_exist(&self, token: &str) -> bool {
        let cf = self.db.cf_handle(TOKEN_CF);
        self.db.get(cf, token).is_ok()
    }

    pub fn delete_token(&self, token: &str) -> Result<(), Error> {
        let cf = self.db.cf_handle(TOKEN_CF);

        self.delete_notifications_by_token(token)?;
        self.delete_addresses_by_token(token)?;

        self.db.delete(cf, token)
    }

    pub fn create_address(&self, token: &str, address: &str) -> Result<(), Error> {
        let cf = self.db.cf_handle(ADDRESS_CF);

        let mut tokens: Vec<String> = Vec::new();

        if let Ok(result) = self.db.get_serialized::<&str, Address>(cf.clone(), address) {
            tokens = result.tokens;

            if !tokens.contains(&token.to_string()) {
                tokens.push(token.to_string());

                let value: Address = Address { tokens };

                let _ = self.db.put_serialized(cf, address, &value);
            }
        } else {
            tokens.push(token.to_string());

            let value: Address = Address { tokens };

            let _ = self.db.put_serialized(cf, address, &value);
        }

        Ok(())
    }

    pub fn get_address<T>(&self, address: T) -> Result<Address, Error>
    where
        T: AsRef<[u8]>,
    {
        let cf = self.db.cf_handle(ADDRESS_CF);
        self.db.get_serialized(cf, address)
    }

    pub fn get_addresses_by_token(&self, token: &str) -> Result<Vec<String>, Error> {
        let cf = self.db.cf_handle(ADDRESS_CF);

        let mut addresses: Vec<String> = Vec::new();
        let collection: HashMap<String, Address> = self.db.iterator_serialized::<Address>(cf)?;

        collection.into_iter().for_each(|(key, value)| {
            if value.tokens.contains(&token.to_string()) {
                addresses.push(key);
            }
        });

        Ok(addresses)
    }

    pub fn delete_address(&self, token: &str, address: &str) -> Result<(), Error> {
        let cf = self.db.cf_handle(ADDRESS_CF);

        if let Ok(result) = self.db.get_serialized::<&str, Address>(cf.clone(), address) {
            let mut tokens: Vec<String> = result.tokens;

            if tokens.contains(&token.to_string()) {
                tokens.retain(|x| x != &token.to_string());

                if tokens.is_empty() {
                    let _ = self.db.delete(self.db.cf_handle(ADDRESS_CF), address);
                } else {
                    let value: Address = Address { tokens };
                    let _ = self.db.put_serialized(cf, address, &value);
                }
            }
        }

        Ok(())
    }

    pub fn delete_addresses_by_token(&self, token: &str) -> Result<(), Error> {
        let cf = self.db.cf_handle(ADDRESS_CF);

        let collection: HashMap<String, Address> = self.db.iterator_serialized::<Address>(cf)?;

        collection.into_iter().for_each(|(key, mut value)| {
            if value.tokens.contains(&token.to_string()) {
                value.tokens.retain(|x| x != &token.to_string());

                if value.tokens.is_empty() {
                    let _ = self.db.delete(self.db.cf_handle(ADDRESS_CF), key.as_str());
                } else {
                    let _ =
                        self.db
                            .put_serialized(self.db.cf_handle(ADDRESS_CF), key.as_str(), &value);
                }
            }
        });

        Ok(())
    }

    pub fn create_notification(
        &self,
        token: &str,
        address: &str,
        txid: &str,
        txtype: &str,
        amount: u64,
        confirmed: bool,
    ) -> Result<(), Error> {
        let cf = self.db.cf_handle(NOTIFICATION_CF);

        let key: &str =
            &util::hash::sha512(format!("{}:{}:{}:{}", token, txid, txtype, amount))[..32];
        let value: Notification = Notification {
            id: key.into(),
            token: token.into(),
            address: address.into(),
            txid: txid.into(),
            txtype: txtype.into(),
            amount,
            confirmed,
            timestamp: util::timestamp(),
        };

        self.db.put_serialized(cf, key, &value)
    }

    pub fn get_notifications(&self) -> Result<Vec<Notification>, Error> {
        let cf = self.db.cf_handle(NOTIFICATION_CF);
        let collection = self.db.iterator_serialized::<Notification>(cf)?;

        Ok(collection.values().cloned().collect())
    }

    pub fn delete_notifications_by_token(&self, token: &str) -> Result<(), Error> {
        let cf = self.db.cf_handle(NOTIFICATION_CF);

        let collection: HashMap<String, Notification> =
            self.db.iterator_serialized::<Notification>(cf)?;

        collection.into_iter().for_each(|(key, value)| {
            if value.token == *token {
                let _ = self
                    .db
                    .delete(self.db.cf_handle(NOTIFICATION_CF), key.as_str());
            }
        });

        Ok(())
    }

    pub fn delete_notifications_by_token_and_ids(
        &self,
        token: &str,
        ids: Vec<String>,
    ) -> Result<(), Error> {
        let cf = self.db.cf_handle(NOTIFICATION_CF);

        let collection: HashMap<String, Notification> =
            self.db.iterator_serialized::<Notification>(cf)?;

        collection.into_iter().for_each(|(key, value)| {
            if value.token == *token && ids.contains(&key) {
                let _ = self
                    .db
                    .delete(self.db.cf_handle(NOTIFICATION_CF), key.as_str());
            }
        });

        Ok(())
    }

    pub fn delete_notifications_by_token_and_address(
        &self,
        token: &str,
        address: &str,
    ) -> Result<(), Error> {
        let cf = self.db.cf_handle(NOTIFICATION_CF);

        let collection: HashMap<String, Notification> =
            self.db.iterator_serialized::<Notification>(cf)?;

        collection.into_iter().for_each(|(key, value)| {
            if value.token == *token && value.address == *address {
                let _ = self
                    .db
                    .delete(self.db.cf_handle(NOTIFICATION_CF), key.as_str());
            }
        });

        Ok(())
    }

    pub fn delete_address_and_notifications(
        &self,
        token: &str,
        address: &str,
    ) -> Result<(), Error> {
        self.delete_notifications_by_token_and_address(token, address)?;

        self.delete_address(token, address)
    }

    pub fn get_notifications_by_token(&self, token: &str) -> Result<Vec<Notification>, Error> {
        let cf = self.db.cf_handle(NOTIFICATION_CF);

        let mut notifications: Vec<Notification> = Vec::new();
        let collection = self.db.iterator_serialized::<Notification>(cf)?;

        collection.into_iter().for_each(|(_, value)| {
            if value.token == *token {
                notifications.push(value);
            }
        });

        Ok(notifications)
    }

    pub fn set_mempool_tx_cached(&self, txid: &str) -> Result<(), Error> {
        let cf = self.db.cf_handle(MEMPOOL_CF);
        let value: Mempool = Mempool {
            timestamp: util::timestamp(),
        };

        match self.db.put_serialized(cf, txid, &value) {
            Ok(_) => {
                log::trace!("Tx {} cached", txid);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    pub fn is_mempool_tx_cached(&self, txid: &str) -> bool {
        let cf = self.db.cf_handle(MEMPOOL_CF);
        self.db.get(cf, txid).is_ok()
    }

    pub fn remove_mempool_tx_cached(&self, txid: &str) -> Result<(), Error> {
        let cf = self.db.cf_handle(MEMPOOL_CF);
        match self.db.delete(cf, txid) {
            Ok(_) => {
                log::trace!("Removed {} from cache", txid);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    pub fn get_mempool_txs_cached(&self) -> Result<HashMap<String, Mempool>, Error> {
        let cf = self.db.cf_handle(MEMPOOL_CF);
        let collection: HashMap<String, Mempool> = self.db.iterator_serialized::<Mempool>(cf)?;

        Ok(collection)
    }
}

impl Drop for CoreStore {
    fn drop(&mut self) {
        log::trace!("Closing Database");
    }
}
