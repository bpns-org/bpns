// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

pub mod api;
pub mod bitcoin;
pub mod db;

use self::bitcoin::Bitcoin;
use self::db::{cleaner::Cleaner, CoreStore};

use crate::CONFIG;

lazy_static! {
    pub(crate) static ref STORE: CoreStore = CoreStore::open(&CONFIG.core.db_path).unwrap();
}

pub struct Core;

impl Core {
    pub fn run() {
        Cleaner::run();
        Bitcoin::run();
    }
}
