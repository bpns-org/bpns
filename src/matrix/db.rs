// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::common::db::{Error, Store};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub device_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub room_id: String,
    pub token: String,
}

#[derive(Clone)]
pub struct MatrixStore {
    pub db: Arc<Store>,
}

const USER_CF: &str = "user";
const SESSION_CF: &str = "session";

const COLUMN_FAMILIES: &[&str] = &[USER_CF, SESSION_CF];

impl MatrixStore {
    pub fn open(path: &Path) -> Result<Self, Error> {
        let db = Store::open(path, COLUMN_FAMILIES)?;

        let store = Self { db: Arc::new(db) };
        Ok(store)
    }

    pub fn create_session(
        &self,
        user_id: &str,
        access_token: &str,
        device_id: &str,
    ) -> Result<(), Error> {
        let value: Session = Session {
            access_token: access_token.into(),
            device_id: device_id.into(),
        };

        self.db
            .put_serialized(self.db.cf_handle(SESSION_CF), user_id, &value)
    }

    pub fn session_exist(&self, user_id: &str) -> bool {
        self.db.get(self.db.cf_handle(SESSION_CF), user_id).is_ok()
    }

    pub fn get_session(&self, user_id: &str) -> Result<Session, Error> {
        self.db
            .get_serialized(self.db.cf_handle(SESSION_CF), user_id)
    }

    /* pub fn delete_session(&self, user_id: &str) -> Result<(), Error> {
        self.db.delete(self.db.cf_handle(SESSION_CF), user_id)
    } */

    pub fn create_user(&self, user_id: &str, room_id: &str, token: &str) -> Result<(), Error> {
        let value: User = User {
            room_id: room_id.into(),
            token: token.into(),
        };

        self.db
            .put_serialized(self.db.cf_handle(USER_CF), user_id, &value)
    }

    pub fn user_exist(&self, user_id: &str) -> bool {
        self.db.get(self.db.cf_handle(USER_CF), user_id).is_ok()
    }

    pub fn user_with_room_exist(&self, user_id: &str, room_id: &str) -> bool {
        if let Ok(user) = self
            .db
            .get_serialized::<&str, User>(self.db.cf_handle(USER_CF), user_id)
        {
            return user.room_id.as_str() == room_id;
        }

        false
    }

    pub fn delete_user(&self, user_id: &str) -> Result<(), Error> {
        self.db.delete(self.db.cf_handle(USER_CF), user_id)
    }

    pub fn get_user(&self, user_id: &str) -> Result<User, Error> {
        self.db.get_serialized(self.db.cf_handle(USER_CF), user_id)
    }

    pub fn get_users(&self) -> Result<Vec<User>, Error> {
        let collection = self
            .db
            .iterator_serialized::<User>(self.db.cf_handle(USER_CF))?;

        Ok(collection.values().cloned().collect())
    }
}

impl Drop for MatrixStore {
    fn drop(&mut self) {
        log::trace!("Closing Database");
    }
}
