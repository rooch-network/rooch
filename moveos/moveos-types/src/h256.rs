// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use fastcrypto::hash::{HashFunction, Sha256};
use once_cell::sync::Lazy;
pub use primitive_types::H256;
use serde::{Deserialize, Serializer};
use std::str::FromStr;
use tiny_keccak::{Hasher, Sha3};

pub const LENGTH: usize = 32;

pub fn sha3_256_of(buffer: &[u8]) -> H256 {
    let mut sha3 = Sha3::v256();
    sha3.update(buffer);
    let mut hash = [0u8; LENGTH];
    sha3.finalize(&mut hash);
    H256(hash)
}

pub fn sha2_256_of(buffer: &[u8]) -> H256 {
    let data = Sha256::digest(buffer);
    H256(data.digest)
}

pub fn serialize<S>(hash: &H256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if serializer.is_human_readable() {
        serializer.serialize_str(&hash.to_string())
    } else {
        serializer.serialize_newtype_struct("H256", &hash.0)
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<H256, D::Error>
where
    D: serde::Deserializer<'de>,
{
    if deserializer.is_human_readable() {
        let s = String::deserialize(deserializer)?;
        H256::from_str(&s).map_err(serde::de::Error::custom)
    } else {
        #[derive(::serde::Deserialize)]
        #[serde(rename = "H256")]
        struct Value([u8; LENGTH]);

        let value = Value::deserialize(deserializer)?;
        Ok(H256(value.0))
    }
}

pub fn create_literal_hash(word: &str) -> H256 {
    let mut s = word.as_bytes().to_vec();
    assert!(s.len() <= LENGTH);
    s.resize(LENGTH, 0);
    H256::from_slice(&s)
}

/// Placeholder hash of `Accumulator`.
pub static ACCUMULATOR_PLACEHOLDER_HASH: Lazy<H256> =
    Lazy::new(|| create_literal_hash("ACCUMULATOR_PLACEHOLDER_HASH"));
