// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use crate::CONFIG;

mod rest;

pub struct Server;

impl Server {
    pub fn run() {
        if CONFIG.server.enabled {
            rest::run();
        }
    }
}
