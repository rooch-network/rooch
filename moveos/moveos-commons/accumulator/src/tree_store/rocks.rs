// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{AccumulatorNode, AccumulatorTreeStore};
use moveos_types::h256::H256;
use rocksdb::DB;

pub struct RocksAccumulatorStore {
    node_store: DB,
    cf_name: String,
}

impl RocksAccumulatorStore {
    pub fn new(node_store: DB, cf_name: String) -> Self {
        Self {
            node_store,
            cf_name,
        }
    }
}

impl AccumulatorTreeStore for RocksAccumulatorStore {
    fn get_node(&self, hash: H256) -> anyhow::Result<Option<AccumulatorNode>> {
        let cf = self.node_store.cf_handle(&self.cf_name).unwrap();
        let v = self
            .node_store
            .get_cf(cf, bcs::to_bytes(&hash).unwrap())?
            .unwrap();
        let node: AccumulatorNode = bcs::from_bytes(&v)?;
        Ok(Some(node))
    }

    fn multiple_get(&self, hash_vec: Vec<H256>) -> anyhow::Result<Vec<Option<AccumulatorNode>>> {
        let cf = self.node_store.cf_handle(&self.cf_name).unwrap();
        let mut nodes = Vec::new();
        for hash in hash_vec {
            let v = self.node_store.get_cf(cf, bcs::to_bytes(&hash).unwrap())?;
            if let Some(v) = v {
                let node: AccumulatorNode = bcs::from_bytes(&v)?;
                nodes.push(Some(node));
            } else {
                nodes.push(None);
            }
        }
        Ok(nodes)
    }

    fn save_node(&self, node: AccumulatorNode) -> anyhow::Result<()> {
        let cf = self.node_store.cf_handle(&self.cf_name).unwrap();
        let k = bcs::to_bytes(&node.hash())?;
        let v = bcs::to_bytes(&node)?;
        self.node_store.put_cf(cf, k, v)?;
        Ok(())
    }

    fn save_nodes(&self, nodes: Vec<AccumulatorNode>) -> anyhow::Result<()> {
        let cf = self.node_store.cf_handle(&self.cf_name).unwrap();
        let mut batch = rocksdb::WriteBatch::default();
        for node in nodes {
            let k = bcs::to_bytes(&node.hash())?;
            let v = bcs::to_bytes(&node)?;
            batch.put_cf(cf, k, v);
        }
        self.node_store.write(batch)?;
        Ok(())
    }

    fn delete_nodes(&self, node_hash_vec: Vec<H256>) -> anyhow::Result<()> {
        let cf = self.node_store.cf_handle(&self.cf_name).unwrap();
        let mut batch = rocksdb::WriteBatch::default();
        for hash in node_hash_vec {
            let k = bcs::to_bytes(&hash)?;
            batch.delete_cf(cf, k);
        }
        self.node_store.write(batch)?;
        Ok(())
    }
}
