// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use sha2::{Digest, Sha512};

use crate::util::convert::bytes_to_hex_string;

pub fn sha512<T>(value: T) -> String
where
    T: AsRef<[u8]>,
{
    let hasher = Sha512::digest(value);
    bytes_to_hex_string(hasher.to_vec())
}
