// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

pub fn bytes_to_hex_string(bytes: Vec<u8>) -> String {
    let mut hash: String = String::new();
    bytes
        .into_iter()
        .for_each(|b| hash.push_str(format!("{:02X}", b).as_str()));

    hash.to_lowercase()
}

pub fn bytes_to_number<T>(bytes: Vec<u8>) -> Option<T>
where
    T: FromStr,
{
    match String::from_utf8(bytes) {
        Ok(result) => match result.parse::<T>() {
            Ok(num) => Some(num),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

pub fn hex_to_bytes(hex_asm: &str) -> Vec<u8> {
    let mut hex_bytes = hex_asm
        .as_bytes()
        .iter()
        .filter_map(|b| match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        })
        .fuse();

    let mut bytes = Vec::new();
    while let (Some(h), Some(l)) = (hex_bytes.next(), hex_bytes.next()) {
        bytes.push(h << 4 | l)
    }
    bytes
}

pub fn vec_to_vec_string<I, T>(iter: I) -> Vec<String>
where
    I: IntoIterator<Item = T>,
    T: Into<String>,
{
    iter.into_iter().map(Into::into).collect()
}
