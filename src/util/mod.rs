// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::{SystemTime, UNIX_EPOCH};

pub mod convert;
pub mod hash;

pub fn timestamp() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(time) => time.as_secs(),
        Err(_) => panic!("Invalid system time"),
    }
}

pub fn generate_entropy() -> String {
    hash::sha512(format!("{}:{}", rand::random::<u128>(), timestamp()).as_str())
}

pub fn generate_token() -> String {
    let mut token: String = String::new();

    for _ in 0..4 {
        token.push_str(&generate_entropy()[0..16]);
    }

    token
}

pub fn is_token(token: &str) -> bool {
    const CHARS: &str = "ABCDEFabcdef0123456789";

    if token.len() < 50 {
        return false;
    }

    for c in token.chars() {
        if !CHARS.contains(c) {
            return false;
        }
    }

    true
}
