// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

mod rest;

pub struct Server;

impl Server {
    pub fn run() {
        rest::run();
    }
}
