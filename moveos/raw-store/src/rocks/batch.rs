// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CodecWriteBatch, WriteOp};
use anyhow::Result;
use moveos_common::utils::to_bytes;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::convert::TryFrom;

#[derive(Debug, Default, Clone)]
pub struct WriteBatch {
    pub rows: Vec<(Vec<u8>, WriteOp<Vec<u8>>)>,
}

impl WriteBatch {
    /// Creates an empty batch.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_rows(rows: Vec<(Vec<u8>, WriteOp<Vec<u8>>)>) -> Self {
        Self { rows }
    }

    /// Adds an insert/update operation to the batch.
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.rows.push((key, WriteOp::Value(value)));
        Ok(())
    }

    /// Adds a delete operation to the batch.
    pub fn delete(&mut self, key: Vec<u8>) -> Result<()> {
        self.rows.push((key, WriteOp::Deletion));
        Ok(())
    }

    ///Clear all operation to the next batch.
    pub fn clear(&mut self) -> Result<()> {
        self.rows.clear();
        Ok(())
    }
}

impl<K, V> TryFrom<CodecWriteBatch<K, V>> for WriteBatch
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    type Error = anyhow::Error;

    fn try_from(batch: CodecWriteBatch<K, V>) -> Result<Self, Self::Error> {
        // let rows: Result<Vec<(Vec<u8>, WriteOp<Vec<u8>>)>> = batch
        let rows: Vec<_> = batch
            .into_iter()
            .map(|(key, op)| (to_bytes(&key).unwrap(), op.into_raw_op().unwrap()))
            .collect();
        Ok(WriteBatch::new_with_rows(rows))
    }
}
