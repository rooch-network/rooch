// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{jellyfish_merkle::hash::SMTHash, HashValue};
use anyhow::Result;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{
    de::{self, DeserializeOwned},
    Deserialize, Serialize,
};
use std::{cell::Cell, fmt};

pub trait Key: std::cmp::Ord + Clone + EncodeToObject + DecodeToObject {}

impl<T: std::cmp::Ord + Clone + EncodeToObject + DecodeToObject> Key for T {}

pub trait Value: Clone + EncodeToObject + DecodeToObject {}

impl<T: Clone + Serialize + EncodeToObject + DecodeToObject> Value for T {}

pub trait EncodeToObject {
    fn into_object(self) -> SMTObject<Self>
    where
        Self: std::marker::Sized;
}

pub trait DecodeToObject {
    fn from_raw(raw: Vec<u8>) -> Result<SMTObject<Self>>
    where
        Self: std::marker::Sized;
}

impl<T> EncodeToObject for T
where
    T: Serialize,
{
    fn into_object(self) -> SMTObject<Self> {
        SMTObject::from_origin(self)
    }
}

impl<T> DecodeToObject for T
where
    T: DeserializeOwned,
{
    fn from_raw(raw: Vec<u8>) -> Result<SMTObject<Self>> {
        SMTObject::from_raw(raw)
    }
}

#[derive(Clone)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct SMTObject<T> {
    pub origin: T,
    pub raw: Vec<u8>,
    cached_hash: Cell<Option<HashValue>>,
}

impl<T> SMTObject<T> {
    pub fn new(origin: T, raw: Vec<u8>) -> Self {
        SMTObject {
            origin,
            raw,
            cached_hash: Cell::new(None),
        }
    }

    /// A helper constructor for tests which allows passing in a precomputed hash.
    pub(crate) fn new_for_test(origin: T, raw: Vec<u8>, hash: HashValue) -> Self {
        SMTObject {
            origin,
            raw,
            cached_hash: Cell::new(Some(hash)),
        }
    }

    pub fn from_origin(origin: T) -> Self
    where
        T: Serialize,
    {
        let raw = bcs::to_bytes(&origin).expect("serialize should not fail");
        SMTObject {
            origin,
            raw,
            cached_hash: Cell::new(None),
        }
    }

    pub fn from_raw(raw: Vec<u8>) -> Result<Self>
    where
        T: DeserializeOwned,
    {
        let origin = bcs::from_bytes(&raw)?;
        Ok(SMTObject {
            origin,
            raw,
            cached_hash: Cell::new(None),
        })
    }
}

impl<T> PartialOrd for SMTObject<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.raw.cmp(&other.raw))
    }
}

impl<T> Ord for SMTObject<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<T> Eq for SMTObject<T> {}

impl<T> PartialEq for SMTObject<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<T> fmt::Debug for SMTObject<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Raw {{ \n \
             raw: 0x{} \n \
             }}",
            hex::encode(&self.raw),
        )
    }
}

impl<T> Serialize for SMTObject<T> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.raw)
    }
}

impl<'de, T> Deserialize<'de> for SMTObject<T>
where
    T: DecodeToObject,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = <Vec<u8>>::deserialize(deserializer)?;
        T::from_raw(raw).map_err(de::Error::custom)
    }
}

impl<T> AsRef<[u8]> for SMTObject<T> {
    fn as_ref(&self) -> &[u8] {
        &self.raw
    }
}

impl<T> AsRef<T> for SMTObject<T> {
    fn as_ref(&self) -> &T {
        &self.origin
    }
}

impl<T> From<T> for SMTObject<T>
where
    T: EncodeToObject,
{
    fn from(origin: T) -> SMTObject<T> {
        origin.into_object()
    }
}

impl<T> SMTHash for SMTObject<T> {
    fn merkle_hash(&self) -> HashValue {
        match self.cached_hash.get() {
            Some(hash) => hash,
            None => {
                let hash = HashValue::sha3_256_of(&self.raw);
                self.cached_hash.set(Some(hash));
                hash
            }
        }
    }
}
