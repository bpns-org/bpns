// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

mod bot;
mod db;
mod util;

use db::MatrixStore;

use crate::{common::thread, CONFIG};

lazy_static! {
    pub(crate) static ref MATRIX_STORE: MatrixStore =
        MatrixStore::open(&CONFIG.matrix.db_path).unwrap();
}

#[tokio::main]
async fn run() {
    bot::Bot::run().await.unwrap();
}

pub struct Matrix;

impl Matrix {
    pub fn run() {
        thread::spawn("matrix_bot", {
            move || {
                run();
            }
        });
    }
}

impl Drop for Matrix {
    fn drop(&mut self) {
        if thread::panicking() {
            std::process::exit(0x1);
        }
    }
}
