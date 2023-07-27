// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod statedb;

use anyhow::Result;
use moveos_types::h256::H256;
use raw_store::{CodecKVStore, CodecWriteBatch};
use smt::NodeStore;
use std::collections::BTreeMap;

use crate::STATE_NODE_PREFIX_NAME;
use raw_store::derive_store;

derive_store!(NodeDBStore, H256, Vec<u8>, STATE_NODE_PREFIX_NAME);

impl NodeStore for NodeDBStore {
    fn get(&self, hash: &H256) -> Result<Option<Vec<u8>>> {
        self.kv_get(*hash)
    }

    fn put(&self, key: H256, node: Vec<u8>) -> Result<()> {
        self.kv_put(key, node)
    }

    fn write_nodes(&self, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()> {
        let batch = CodecWriteBatch::new_puts(nodes.into_iter().collect());
        self.write_batch(batch)
    }
}
