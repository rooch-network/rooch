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

/// Store-optimized H256 that always serializes to raw 32 bytes
/// This is used as a database key to avoid the string serialization overhead
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct StoreKeyH256([u8; LENGTH]);

impl StoreKeyH256 {
    pub fn new(hash: H256) -> Self {
        Self(hash.0)
    }

    pub fn as_h256(&self) -> H256 {
        H256(self.0)
    }

    pub fn as_bytes(&self) -> &[u8; LENGTH] {
        &self.0
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != LENGTH {
            return Err("Invalid length for StoreKeyH256");
        }
        let mut arr = [0u8; LENGTH];
        arr.copy_from_slice(bytes);
        Ok(Self(arr))
    }
}

impl From<H256> for StoreKeyH256 {
    fn from(hash: H256) -> Self {
        Self(hash.0)
    }
}

impl From<StoreKeyH256> for H256 {
    fn from(store_hash: StoreKeyH256) -> Self {
        H256(store_hash.0)
    }
}

impl From<&[u8; LENGTH]> for StoreKeyH256 {
    fn from(bytes: &[u8; LENGTH]) -> Self {
        Self(*bytes)
    }
}

// Always serialize as raw 32 bytes for optimal database storage
impl serde::Serialize for StoreKeyH256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct("StoreKeyH256", &self.0)
    }
}

impl<'de> serde::Deserialize<'de> for StoreKeyH256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(::serde::Deserialize)]
        #[serde(rename = "StoreKeyH256")]
        struct Value([u8; LENGTH]);

        let value = Value::deserialize(deserializer)?;
        Ok(Self(value.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_key_h256_serialization() {
        let hash = H256::random();
        let store_hash = StoreKeyH256::new(hash);

        // Test BCS serialization (should be 32 bytes)
        let bcs_bytes = bcs::to_bytes(&store_hash).unwrap();
        assert_eq!(bcs_bytes.len(), 32, "BCS should be 32 bytes");
        assert_eq!(bcs_bytes, hash.as_bytes(), "BCS should be raw bytes");

        // Test round-trip
        let deserialized: StoreKeyH256 = bcs::from_bytes(&bcs_bytes).unwrap();
        assert_eq!(
            deserialized.as_h256(),
            hash,
            "Round-trip should preserve H256"
        );

        // Test that it always serializes to raw 32 bytes, even for JSON
        let json_bytes = serde_json::to_vec(&store_hash).unwrap();
        println!("JSON serialization bytes: {:?}", json_bytes);
        // JSON format will be different, but the underlying H256 data should be preserved
        let json_deserialized: StoreKeyH256 = serde_json::from_slice(&json_bytes).unwrap();
        assert_eq!(
            json_deserialized.as_h256(),
            hash,
            "JSON round-trip should preserve H256"
        );
    }

    #[test]
    fn test_store_key_h256_conversions() {
        let hash = H256::random();
        let store_hash = StoreKeyH256::new(hash);

        // Test conversions
        let back_to_h256: H256 = store_hash.into();
        assert_eq!(back_to_h256, hash);

        let from_h256: StoreKeyH256 = hash.into();
        assert_eq!(from_h256.as_h256(), hash);
    }
}
