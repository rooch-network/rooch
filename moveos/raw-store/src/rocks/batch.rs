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

    /// Clear all operation to the next batch.
    pub fn clear(&mut self) -> Result<()> {
        self.rows.clear();
        Ok(())
    }

    pub fn size_in_bytes(&self) -> usize {
        let mut batch_size: usize = 0;
        for (k, op) in self.rows.iter() {
            batch_size += k.len();
            match op {
                WriteOp::Value(v) => batch_size += v.len(),
                WriteOp::Deletion => {}
            }
        }
        batch_size
    }

    pub fn extend(&mut self, other: &WriteBatch) {
        self.rows.extend_from_slice(&other.rows);
    }
}

impl<K, V> TryFrom<CodecWriteBatch<K, V>> for WriteBatch
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    type Error = anyhow::Error;

    fn try_from(batch: CodecWriteBatch<K, V>) -> Result<Self, Self::Error> {
        let rows: Vec<_> = batch
            .into_iter()
            .map(|(key, op)| (to_bytes(&key).unwrap(), op.into_raw_op().unwrap()))
            .collect();
        Ok(WriteBatch::new_with_rows(rows))
    }
}

#[derive(Debug, Default, Clone)]
/// WriteBatchCF is a WriteBatch with a column family name.
pub struct WriteBatchCF {
    pub batch: WriteBatch,
    pub cf_name: String,
}

impl WriteBatchCF {
    /// Creates an empty batch.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_rows(rows: Vec<(Vec<u8>, WriteOp<Vec<u8>>)>, cf_name: String) -> Self {
        Self {
            batch: WriteBatch::new_with_rows(rows),
            cf_name,
        }
    }

    /// Adds an insert/update operation to the batch.
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.batch.put(key, value)
    }

    /// Adds a delete operation to the batch.
    pub fn delete(&mut self, key: Vec<u8>) -> Result<()> {
        self.batch.delete(key)
    }

    /// Clear all operation to the next batch.
    pub fn clear(&mut self) -> Result<()> {
        self.batch.clear()
    }

    pub fn size_in_bytes(&self) -> usize {
        self.batch.size_in_bytes()
    }

    pub fn extend(&mut self, other: &WriteBatch) {
        self.batch.extend(other);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch() {
        let mut batch = WriteBatch::new();
        batch.put(b"key1".to_vec(), b"value1".to_vec()).unwrap();
        batch.put(b"key2".to_vec(), b"value2".to_vec()).unwrap();
        batch.delete(b"key1".to_vec()).unwrap();
        assert_eq!(batch.rows.len(), 3);
        assert_eq!(batch.rows[0].0, b"key1");
        assert_eq!(batch.rows[1].0, b"key2");
        assert_eq!(batch.rows[2].0, b"key1");

        let mut batch2 = WriteBatch::new();
        batch2.put(b"key3".to_vec(), b"value3".to_vec()).unwrap();
        batch2.put(b"key4".to_vec(), b"value4".to_vec()).unwrap();
        batch2.delete(b"key3".to_vec()).unwrap();

        batch.extend(&batch2);
        assert_eq!(batch.rows.len(), 6);
        assert_eq!(batch.rows[3].0, b"key3");
        assert_eq!(batch.rows[4].0, b"key4");
        assert_eq!(batch.rows[5].0, b"key3");
    }
}
