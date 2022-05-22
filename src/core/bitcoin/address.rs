// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

extern crate bitcoin;

use std::str::FromStr;

use bdk::wallet::AddressIndex::Peek;
use bdk::{database::MemoryDatabase, Wallet};
use bitcoin::{
    blockdata::{opcodes, script},
    network::constants::Network,
    secp256k1::{ffi::types::AlignedType, AllPreallocated, Secp256k1},
    util::{
        address::Address,
        base58,
        bip32::{ChildNumber, ExtendedPubKey},
    },
    PublicKey,
};

use crate::util;

const NETWORK: Network = bitcoin::Network::Bitcoin;

pub fn is_address(address: &str) -> bool {
    Address::from_str(address).is_ok()
}

fn convert_to_xpub(public_key: &str) -> Option<Vec<u8>> {
    let data = match base58::from_check(public_key) {
        Ok(result) => result,
        Err(_) => return None,
    };
    Some([util::convert::hex_to_bytes("0488b21e"), data[4..].to_vec()].concat())
}

pub fn is_public_key(public_key: &str) -> bool {
    convert_to_xpub(public_key).is_some()
}

pub fn from_descriptor(descriptor: &str, from_index: u32, to_index: u32) -> Option<Vec<String>> {
    let wallet = Wallet::new(descriptor, None, NETWORK, MemoryDatabase::default()).unwrap();

    let mut addresses: Vec<String> = Vec::new();

    for index in from_index..=to_index {
        let address: String = wallet.get_address(Peek(index)).unwrap().to_string();
        addresses.push(address);
    }

    Some(addresses)
}

pub fn from_singlesig(
    public_key: &str,
    from_index: u32,
    to_index: u32,
    is_change: bool,
) -> Option<Vec<String>> {
    if !is_public_key(public_key) {
        return None;
    }

    let mut buf: Vec<AlignedType> = vec![AlignedType::zeroed(); Secp256k1::preallocate_size()];
    let secp: Secp256k1<AllPreallocated> = Secp256k1::preallocated_new(&mut buf).unwrap();

    let prefix: &str = &public_key[0..4];
    let xpub = ExtendedPubKey::decode(&convert_to_xpub(public_key).unwrap()).unwrap();

    let change = ChildNumber::from_normal_idx(if is_change { 1 } else { 0 }).unwrap();

    let mut addresses: Vec<String> = Vec::new();

    for index in from_index..=to_index {
        let index = ChildNumber::from_normal_idx(index).unwrap();

        let public_key: PublicKey = xpub
            .derive_pub(&secp, &vec![change, index])
            .unwrap()
            .public_key;

        match prefix {
            "xpub" => addresses.push(Address::p2pkh(&public_key, NETWORK).to_string()),
            "ypub" => {
                if let Ok(address) = Address::p2shwpkh(&public_key, NETWORK) {
                    addresses.push(address.to_string())
                }
            }
            "zpub" => {
                if let Ok(address) = Address::p2wpkh(&public_key, NETWORK) {
                    addresses.push(address.to_string())
                }
            }
            _ => (),
        };
    }

    Some(addresses)
}

pub struct Multisig {
    pub script_type: String,
    pub required_signatures: i64,
    pub xpub_keys: Vec<ExtendedPubKey>,
}

#[derive(Debug)]
pub enum MultisigError {
    InvalidScriptType,
    InvalidSignatureNumber,
    InvalidXPubKeysNumber,
    PublicKeysEmpty,
    PublicKeysWithDifferentScriptTypes,
}

impl Multisig {
    pub fn new(
        script_type: &str,
        required_signatures: i64,
        public_keys: &[String],
    ) -> Result<Self, MultisigError> {
        if script_type != "p2wsh" && script_type != "p2shwsh" && script_type != "p2sh" {
            return Err(MultisigError::InvalidScriptType);
        }

        if public_keys.is_empty() {
            return Err(MultisigError::PublicKeysEmpty);
        }

        let public_keys_len: i64 = public_keys.len() as i64;

        if required_signatures > public_keys_len {
            return Err(MultisigError::InvalidSignatureNumber);
        }

        // Check if prefix of public keys are the same.
        if !&public_keys
            .iter()
            .all(|item| item[0..4] == public_keys[0][0..4])
        {
            return Err(MultisigError::PublicKeysWithDifferentScriptTypes);
        }

        let mut xpub_keys: Vec<ExtendedPubKey> = Vec::new();

        public_keys.iter().for_each(|public_key| {
            if let Some(xpub_key) = convert_to_xpub(public_key.as_str()) {
                xpub_keys.push(ExtendedPubKey::decode(&xpub_key).unwrap());
            }
        });

        if xpub_keys.len() as i64 != public_keys_len {
            return Err(MultisigError::InvalidXPubKeysNumber);
        }

        Ok(Self {
            script_type: script_type.into(),
            required_signatures,
            xpub_keys,
        })
    }

    pub fn derive(
        &self,
        secp: &Secp256k1<AllPreallocated>,
        index: u32,
        is_change: bool,
    ) -> Option<String> {
        let change = ChildNumber::from_normal_idx(if is_change { 1 } else { 0 }).unwrap();

        let index = ChildNumber::from_normal_idx(index).unwrap();

        let mut pubkeys: Vec<PublicKey> = Vec::new();

        self.xpub_keys.iter().for_each(|xpub_key| {
            let new_public_key: PublicKey = xpub_key
                .derive_pub(secp, &vec![change, index])
                .unwrap()
                .public_key;
            pubkeys.push(new_public_key);
        });

        pubkeys.sort();

        let mut script_builder = script::Builder::new();
        script_builder = script_builder.push_int(self.required_signatures);
        pubkeys
            .iter()
            .for_each(|pubkey| script_builder = script_builder.clone().push_key(pubkey));
        script_builder = script_builder.push_int(self.xpub_keys.len() as i64);
        script_builder = script_builder.push_opcode(opcodes::all::OP_CHECKMULTISIG);

        let script: script::Script = script_builder.into_script();

        match self.script_type.as_str() {
            "p2wsh" => Some(Address::p2wsh(&script, NETWORK).to_string()),
            "p2shwsh" => Some(Address::p2shwsh(&script, NETWORK).to_string()),
            "p2sh" => Some(Address::p2sh(&script, NETWORK).to_string()),
            _ => None,
        }
    }
}

/* pub fn multiple_from_multisig(
    script_type: &str,
    required_signatures: i64,
    public_keys: &[String],
    from_index: u32,
    to_index: u32,
    is_change: bool,
) -> Option<Vec<String>> {
    if script_type != "p2wsh" && script_type != "p2shwsh" && script_type != "p2sh" {
        return None;
    }

    if public_keys.is_empty() {
        return None;
    }

    let mut buf: Vec<AlignedType> = Vec::new();
    buf.resize(Secp256k1::preallocate_size(), AlignedType::zeroed());
    let secp: Secp256k1<AllPreallocated> = Secp256k1::preallocated_new(buf.as_mut_slice()).unwrap();

    let public_keys_len: i64 = public_keys.len() as i64;

    if required_signatures > public_keys_len {
        return None;
    }

    // Check if prefix of public keys are the same.
    if !&public_keys
        .iter()
        .all(|item| item[0..4] == public_keys[0][0..4])
    {
        return None;
    }

    let mut xpub_keys: Vec<ExtendedPubKey> = Vec::new();

    public_keys.iter().for_each(|public_key| {
        if let Some(xpub_key) = convert_to_xpub(public_key.as_str()) {
            xpub_keys.push(ExtendedPubKey::decode(&xpub_key).unwrap());
        }
    });

    if xpub_keys.len() as i64 != public_keys_len {
        return None;
    }

    let change = ChildNumber::from_normal_idx(if is_change { 1 } else { 0 }).unwrap();

    let addresses: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    (from_index..=to_index).into_par_iter().for_each(|index| {
        let index = ChildNumber::from_normal_idx(index).unwrap();

        let mut pubkeys: Vec<PublicKey> = Vec::new();

        xpub_keys.clone().iter().for_each(|xpub_key| {
            let new_public_key: PublicKey = xpub_key
                .derive_pub(&secp, &vec![change, index])
                .unwrap()
                .public_key;
            pubkeys.push(new_public_key);
        });

        pubkeys.sort();

        let mut script_builder = script::Builder::new();
        script_builder = script_builder.push_int(required_signatures);
        pubkeys
            .iter()
            .for_each(|pubkey| script_builder = script_builder.clone().push_key(pubkey));
        script_builder = script_builder.push_int(public_keys_len);
        script_builder = script_builder.push_opcode(opcodes::all::OP_CHECKMULTISIG);

        let script: script::Script = script_builder.into_script();

        let mut addresses = addresses.lock().unwrap();

        match script_type {
            "p2wsh" => addresses.push(Address::p2wsh(&script, NETWORK).to_string()),
            "p2shwsh" => addresses.push(Address::p2shwsh(&script, NETWORK).to_string()),
            "p2sh" => addresses.push(Address::p2sh(&script, NETWORK).to_string()),
            _ => (),
        };
    });

    Some(addresses.clone().lock().unwrap().to_vec())
} */

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid_addresses() {
        assert!(is_address("12dRugNcdxK39288NjcDV4GX7rMsKCGn6B"));
        assert!(is_address("3NtwbVymuhJ9S7zbovytrysHJBQQQQ81B2"));
        assert!(is_address("bc1qe7f3h290cyf55ccf62d80kr43h49lya5ac9pt2"));
        assert!(is_address(
            "bc1q7ug4w4as2sefar89q057hnmxkakp58a25535ttlmurn6cncs8tms4e7gp2"
        ));
    }

    #[test]
    fn invalid_addresses() {
        assert!(!is_address("test"));
        assert!(!is_address("12dRugNcdxK39288NjcDV4GX7rMsKCPf7C"));
        assert!(!is_address("3NtwbVymuhJ9S7zbovytrysHJBABCDE8ev"));
        assert!(!is_address("bc1qe7f3h290cyf55ccf62d80kr43h49lya5ac9me5"));
        assert!(!is_address(
            "bc1q7ug4w4as2sefar89q057hnmxkakp58a25535ttlmurn6cncs8tms4e7f8f"
        ));
    }

    #[test]
    fn valid_pubkey() {
        assert!(is_public_key("xpub6Bwfu1R7aLXwczEjjx9pwFzyssVmfEgkurM7vtHk9GKSaRL4PQYigqRKku6d9RtaNyuSXLFCZuNpLKzm3jWEUERb5JtGgdr3PWQnyhL6Ruw"));
    }

    #[test]
    fn invalid_pubkey() {
        assert!(!is_public_key("xpub6Bwfu1R7aLXwczEjjx9pwFzyssVmfEgkurM7vtHk9GKSaRL4PQYigqRKku6d9RtaNyuSXLFCZuNpLKzm3jWEUERb5JtGgdr3PWQnyhL6Ruv"));
    }

    #[test]
    fn derive_address_from_descriptor() {
        let pubkey = "zpub6rQRC2gB41AGDJXMzuahP6DF1jkEys8nRUsYQktH6gUCqDFaExbNyuZetmfFyrSsJdvEio4jYRRhkLxeFPttxJD2R5R2GoBCpKQUtSaAdsu";
        let xpubkey: String = base58::check_encode_slice(&convert_to_xpub(pubkey).unwrap());
        let descriptor: &str = &format!("wpkh([9ddc948e/84h/0h/0h]{}/0/*)", xpubkey);

        assert_eq!(
            from_descriptor(descriptor, 0, 1),
            Some(vec![
                "bc1qs8tmhel359z72lrdvwaj90nxs0l8mnu68qxhaq".into(),
                "bc1qnep9s72anmlx86hzt7e2w52jxgys0fredg606y".into()
            ])
        );
    }

    #[test]
    fn derive_address_from_xpub() {
        let pubkey: &str = "xpub6Bwfu1R7aLXwczEjjx9pwFzyssVmfEgkurM7vtHk9GKSaRL4PQYigqRKku6d9RtaNyuSXLFCZuNpLKzm3jWEUERb5JtGgdr3PWQnyhL6Ruw";
        assert_eq!(
            from_singlesig(pubkey, 0, 0, false),
            Some(vec!["1PW7vCjj68jC1T2hSUPw9n7AQUNYuv2rEi".into()])
        );
        assert_eq!(
            from_singlesig(pubkey, 1, 1, false),
            Some(vec!["1CUPYKxa8kYWAuDZoj7qxTr3nbPr3Cm6N4".into()])
        );
    }

    #[test]
    fn derive_address_from_ypub() {
        let pubkey: &str = "ypub6Y24XHMwnhH5NQ5Jr9qDyYGhLFgS5hHp65AqkU3k3xHQdLn9V5M2YWQ8yn7nKB4eQBD5o8XvYoYp1bsi71Wkggo1xeTGpPmQ45ReDxpP9Qq";
        assert_eq!(
            from_singlesig(pubkey, 0, 0, false),
            Some(vec!["3BNmyiCqZUhy4vwRn4Co8CQ6YmKnQRsVSP".into()])
        );
        assert_eq!(
            from_singlesig(pubkey, 3, 3, false),
            Some(vec!["3DGHrFQELq4dnAVg8DHwSLAkXckc3rrVcf".into()])
        );
    }

    #[test]
    fn derive_address_from_zpub() {
        let pubkey: &str = "zpub6s1rSuNVVpH88zXPyXdtCduh8XwyaE9eCBYiCXM29iF9gHpDznAU2F4GeYZe7qi3SwdZ9BJm1gkDD8C3SGp7qnA9D2hJjyFRU8b6EeYnTH9";
        assert_eq!(
            from_singlesig(pubkey, 0, 0, false),
            Some(vec!["bc1quywlxrc3v3x3qxeau9vdz9xle2yzg2wfks05dv".into()])
        );
        assert_eq!(
            from_singlesig(pubkey, 6, 6, false),
            Some(vec!["bc1qak2mkwwwa2u8zu8df95llp8cdz027wu6wr5h3y".into()])
        );
    }

    #[test]
    fn derive_p2sh_multisig_address() {
        let public_keys = vec![
            "xpub6BUn9mzcqwApD9S3WjTABNyYwkQUByRsSNcp8HpoyHTkLNmoJVkdqCWW5Lwk94Fiy5VSb9pbNG6ZuS3peYKSrqYs79VYqoAR5BEt378oJyJ".into(), 
            "xpub6BYAUSRszbAiK2mdj4AmUGj632mHPZtUtMk2k7D2V2c4dBXffMWLcSzuSHErUrxyKDwjXzsgkgZmUZpkTfU9TPTivsN2LCSw32swyNGhCg7".into()
        ];

        let mut buf: Vec<AlignedType> = vec![AlignedType::zeroed(); Secp256k1::preallocate_size()];
        let secp: Secp256k1<AllPreallocated> = Secp256k1::preallocated_new(&mut buf).unwrap();

        let multisig = Multisig::new("p2sh", 2, &public_keys).unwrap();
        let address = multisig.derive(&secp, 0, false);

        assert_eq!(address, Some("3AChTvyFF3cfkUPwDDgSZ3kVxg8CYKup7d".into()));
    }

    #[test]
    fn derive_p2shwsh_multisig_address() {
        let public_keys = vec![
            "Ypub6jJiTo3kTpTzpfyiYF46yNCLUEHjWEY8ndxNUYuaqVBDV4q6aKXFY7AF6uPSffQL3UfYyAyQikWWMtWPrZFCao9j1ciTXXnf57cLbd6yKWi".into(), 
            "Ypub6jGwFfLtT25UfPhuYtiZKyMX3Mg7ntEzsc3y8Ztd6weSpDXAAERdrFPerxgK9xc4YEDeosgjYfRS74C8vGbD5mDt2Dsu3MN8NtX26ix4Ene".into()
        ];

        let mut buf: Vec<AlignedType> = vec![AlignedType::zeroed(); Secp256k1::preallocate_size()];
        let secp: Secp256k1<AllPreallocated> = Secp256k1::preallocated_new(&mut buf).unwrap();

        let multisig = Multisig::new("p2shwsh", 2, &public_keys).unwrap();
        let address = multisig.derive(&secp, 11, false);

        assert_eq!(address, Some("33hHjt4KJeSw4nDu2fWDUQndfa34jpiEhF".into()));
    }

    #[test]
    fn derive_p2wsh_multisig_address() {
        let public_keys = vec![
            "Zpub748ymTifcW1UhCiJHKmXcpRe5AGKsYnbYFyecW7Wbbwm2jghz8SaJ7sNEQMEHovqv3xaHMWCzPFkmRSEqgLNYaiHBtP26KsNDgaF8eRjTWq".into(), 
            "Zpub747CZL1obhcxYuenctciPW6Y2WzMf9eYQuqQbCQGDEqfYFZkMz3gCRB7qGwZifZwqAaQRQDUwed8UztrVp62o2BqTjDKh716UTuMnmtrJoh".into()
        ];

        let mut buf: Vec<AlignedType> = vec![AlignedType::zeroed(); Secp256k1::preallocate_size()];
        let secp: Secp256k1<AllPreallocated> = Secp256k1::preallocated_new(&mut buf).unwrap();

        let multisig = Multisig::new("p2wsh", 2, &public_keys).unwrap();
        let address = multisig.derive(&secp, 3, true);

        assert_eq!(
            address,
            Some("bc1q4qk0ldm63qyl4h6e9ruz6wcvtu2hm7y0y08mg02cy92qwpzq6cvqsdtwzw".into())
        );
    }
}
