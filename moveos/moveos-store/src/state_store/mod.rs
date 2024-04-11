// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod statedb;

use crate::STATE_NODE_PREFIX_NAME;
use anyhow::Result;
use moveos_types::h256::H256;
use raw_store::derive_store;
use raw_store::{CodecKVStore, CodecWriteBatch};
use smt::NodeReader;
use std::collections::BTreeMap;

derive_store!(NodeDBStore, H256, Vec<u8>, STATE_NODE_PREFIX_NAME);

impl NodeDBStore {
    pub fn put(&self, key: H256, node: Vec<u8>) -> Result<()> {
        self.kv_put(key, node)
    }

    pub fn write_nodes(&self, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()> {
        let batch = CodecWriteBatch::new_puts(nodes.into_iter().collect());
        self.write_batch(batch)
    }
}

impl NodeReader for NodeDBStore {
    fn get(&self, hash: &H256) -> Result<Option<Vec<u8>>> {
        self.kv_get(*hash)
    }
}
