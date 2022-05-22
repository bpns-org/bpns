// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

extern crate bitcoin as rust_bitcoin;
use rust_bitcoin::secp256k1::{ffi::types::AlignedType, Error as Secp256k1Error, Secp256k1};

use crate::{
    common,
    core::{bitcoin, db, STORE},
    util,
};

pub struct CoreApi {
    token: String,
}

#[derive(Debug)]
pub enum Error {
    Db(common::db::Error),
    Secp256k1(Secp256k1Error),
    InvalidArgs,
}

impl CoreApi {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.into(),
        }
    }

    pub fn new_push_notification_token() -> String {
        util::generate_token()
    }

    pub fn is_subscribed(&self) -> bool {
        STORE.token_exist(self.token.as_str())
    }

    pub fn subscribe(&self) -> Result<(), Error> {
        if let Err(error) = STORE.create_token(self.token.as_str()) {
            match error {
                common::db::Error::AlreadyExist => return Ok(()),
                _ => return Err(Error::Db(error)),
            }
        }
        Ok(())
    }

    pub fn unsubscribe(&self) -> Result<(), Error> {
        STORE.delete_token(self.token.as_str())?;
        Ok(())
    }

    pub fn notifications(&self) -> Result<Vec<db::Notification>, Error> {
        let notifications = STORE.get_notifications_by_token(self.token.as_str())?;
        Ok(notifications)
    }

    pub fn delete_notification_by_id(&self, id: &str) -> Result<(), Error> {
        STORE.delete_notifications_by_token_and_ids(self.token.as_str(), vec![id.to_string()])?;
        Ok(())
    }

    pub fn delete_all_notifications(&self) -> Result<(), Error> {
        STORE.delete_notifications_by_token(self.token.as_str())?;
        Ok(())
    }

    pub fn addresses(&self) -> Result<Vec<String>, Error> {
        let addresses = STORE.get_addresses_by_token(self.token.as_str())?;
        Ok(addresses)
    }

    pub fn add_addresses(&self, addresses: Vec<String>) {
        addresses.iter().for_each(|address| {
            if bitcoin::address::is_address(address.as_str()) {
                let _ = STORE.create_address(self.token.as_str(), address.as_str());
            }
        });
    }

    pub fn delete_addresses(&self, addresses: Vec<String>) {
        addresses.iter().for_each(|address| {
            if bitcoin::address::is_address(address.as_str()) {
                let _ =
                    STORE.delete_address_and_notifications(self.token.as_str(), address.as_str());
            }
        });
    }

    pub fn add_addresses_from_singlesig(
        &self,
        public_key: &str,
        from_index: u32,
        to_index: u32,
        is_change: bool,
    ) -> Result<(), Error> {
        let addresses =
            bitcoin::address::from_singlesig(public_key, from_index, to_index, is_change);

        match addresses {
            Some(addresses) => {
                self.add_addresses(addresses);
                Ok(())
            }
            None => Err(Error::InvalidArgs),
        }
    }

    pub fn delete_addresses_from_singlesig(
        &self,
        public_key: &str,
        from_index: u32,
        to_index: u32,
        is_change: bool,
    ) -> Result<(), Error> {
        let addresses =
            bitcoin::address::from_singlesig(public_key, from_index, to_index, is_change);

        match addresses {
            Some(addresses) => {
                self.delete_addresses(addresses);
                Ok(())
            }
            None => Err(Error::InvalidArgs),
        }
    }

    pub fn add_addresses_from_multisig(
        &self,
        script_type: &str,
        required_signatures: u8,
        public_keys: &[String],
        from_index: u32,
        to_index: u32,
        is_change: bool,
    ) -> Result<(), Error> {
        let multisig = match bitcoin::address::Multisig::new(
            script_type,
            required_signatures.into(),
            public_keys,
        ) {
            Ok(multisig) => multisig,
            Err(_) => return Err(Error::InvalidArgs),
        };

        let mut buf: Vec<AlignedType> = Vec::new();
        buf.resize(Secp256k1::preallocate_size(), AlignedType::zeroed());
        let secp = Secp256k1::preallocated_new(buf.as_mut_slice())?;

        (from_index..=to_index).into_iter().for_each(|index| {
            if let Some(address) = multisig.derive(&secp, index, is_change) {
                let _ = STORE.create_address(self.token.as_str(), address.as_str());
            }
        });

        Ok(())
    }

    pub fn delete_addresses_from_multisig(
        &self,
        script_type: &str,
        required_signatures: u8,
        public_keys: &[String],
        from_index: u32,
        to_index: u32,
        is_change: bool,
    ) -> Result<(), Error> {
        let multisig = match bitcoin::address::Multisig::new(
            script_type,
            required_signatures.into(),
            public_keys,
        ) {
            Ok(multisig) => multisig,
            Err(_) => return Err(Error::InvalidArgs),
        };

        let mut buf: Vec<AlignedType> = Vec::new();
        buf.resize(Secp256k1::preallocate_size(), AlignedType::zeroed());
        let secp = Secp256k1::preallocated_new(buf.as_mut_slice())?;

        (from_index..=to_index).into_iter().for_each(|index| {
            if let Some(address) = multisig.derive(&secp, index, is_change) {
                let _ = STORE.delete_address(self.token.as_str(), address.as_str());
            }
        });

        Ok(())
    }
}

impl From<common::db::Error> for Error {
    fn from(err: common::db::Error) -> Self {
        Error::Db(err)
    }
}

impl From<Secp256k1Error> for Error {
    fn from(err: Secp256k1Error) -> Self {
        Error::Secp256k1(err)
    }
}
