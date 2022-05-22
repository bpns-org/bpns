// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;

mod common;
mod config;
mod core;
mod logger;
#[cfg(feature = "matrix")]
mod matrix;
#[cfg(feature = "server")]
mod server;
#[cfg(feature = "telegram")]
mod telegram;
mod util;

use crate::{config::Config, core::Core, logger::Logger};

#[cfg(feature = "matrix")]
use matrix::Matrix;
#[cfg(feature = "server")]
use server::Server;
#[cfg(feature = "telegram")]
use telegram::Telegram;

lazy_static! {
    pub(crate) static ref CONFIG: Config = Config::from_args();
}

pub fn run() {
    Logger::init();

    Core::run();

    #[cfg(feature = "matrix")]
    Matrix::run();

    #[cfg(feature = "telegram")]
    Telegram::run();

    #[cfg(feature = "server")]
    Server::run();

    #[cfg(not(feature = "server"))]
    loop {
        common::thread::sleep(60);
    }
}
