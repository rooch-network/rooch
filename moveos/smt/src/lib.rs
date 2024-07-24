// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use function_name::named;
use jellyfish_merkle::hash::SPARSE_MERKLE_PLACEHOLDER_HASH_VALUE;
use jellyfish_merkle::{
    iterator::JellyfishMerkleIterator,
    node_type::{Node, NodeKey},
    JellyfishMerkleTree, TreeReader,
};
use parking_lot::RwLock;
use primitive_types::H256;
use prometheus::Registry;
use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
    sync::Arc,
};

use crate::metrics::SMTMetrics;
pub use jellyfish_merkle::{hash::SPARSE_MERKLE_PLACEHOLDER_HASH, proof::SparseMerkleProof};
pub use smt_object::{DecodeToObject, EncodeToObject, Key, SMTObject, Value};
pub use update_set::UpdateSet;

pub(crate) mod jellyfish_merkle;
pub mod metrics;
mod smt_object;
#[cfg(test)]
pub(crate) mod tests;
mod update_set;

/// MerkeHashType is a trait to indicate the type can be converted to H256.
/// So the type can be used as the key of the Sparse Merkle Tree, and do not need to hash again.

/// Load the tree node binary via hash
pub trait NodeReader {
    fn get(&self, hash: &H256) -> Result<Option<Vec<u8>>>;
}

pub trait NodeWriter {
    fn write_nodes(&self, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()>;
}

impl<K, V, NR> TreeReader<K, V> for NR
where
    NR: NodeReader,
    K: Key,
    V: Value,
{
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node<K, V>>> {
        if node_key == &*SPARSE_MERKLE_PLACEHOLDER_HASH_VALUE {
            return Ok(Some(Node::new_null()));
        }
        //TODO implement a LRU CachedTreeReader to reduce the decode cost
        self.get(&(*node_key).into())?
            .map(|v| Node::<K, V>::decode(&v))
            .transpose()
    }
}

#[derive(Default, Clone)]
pub struct InMemoryNodeStore {
    inner: Arc<RwLock<HashMap<H256, Vec<u8>>>>,
}

impl InMemoryNodeStore {
    pub fn put(&self, key: H256, node: Vec<u8>) -> Result<()> {
        self.inner.write().insert(key, node);
        Ok(())
    }

    pub fn write_nodes(&self, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()> {
        self.inner.write().extend(nodes);
        Ok(())
    }
}

impl From<HashMap<H256, Vec<u8>>> for InMemoryNodeStore {
    fn from(map: HashMap<H256, Vec<u8>>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(map)),
        }
    }
}

impl NodeReader for InMemoryNodeStore {
    fn get(&self, hash: &H256) -> Result<Option<Vec<u8>>> {
        Ok(self.inner.read().get(hash).cloned())
    }
}

impl NodeWriter for InMemoryNodeStore {
    fn write_nodes(&self, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()> {
        InMemoryNodeStore::write_nodes(self, nodes)
    }
}

#[derive(Debug, Clone)]
pub struct TreeChangeSet {
    pub state_root: H256,
    pub nodes: BTreeMap<H256, Vec<u8>>,
}

impl TreeChangeSet {
    pub fn new(state_root: H256, nodes: BTreeMap<H256, Vec<u8>>) -> Self {
        Self { state_root, nodes }
    }
}

/// Sparse Merkle Tree
#[derive(Clone)]
pub struct SMTree<K, V, NR> {
    node_reader: NR,
    key: PhantomData<K>,
    value: PhantomData<V>,
    metrics: Arc<SMTMetrics>,
}

impl<K, V, NR> SMTree<K, V, NR>
where
    K: Key,
    V: Value,
    NR: NodeReader,
{
    /// Construct a new smt tree with a tree reader.
    pub fn new(node_reader: NR, registry: &Registry) -> Self {
        SMTree {
            node_reader,
            key: PhantomData,
            value: PhantomData,
            metrics: Arc::new(SMTMetrics::new(registry)),
        }
    }

    /// Put a kv pair into tree and generate new state_root.
    /// If need to put many kvs, please use `puts` method
    #[named]
    pub fn put(&self, state_root: H256, key: K, value: V) -> Result<TreeChangeSet> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .smt_put_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let result = self.puts(state_root, (key, Some(value)))?;
        let size = result.nodes.values().map(|v| v.len()).sum::<usize>();
        self.metrics
            .smt_put_bytes
            .with_label_values(&[fn_name])
            .observe(size as f64);
        Ok(result)
    }

    /// Remove key_hash's data.
    /// Same as put(K,None)
    #[named]
    pub fn remove(&self, state_root: H256, key: K) -> Result<TreeChangeSet> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .smt_remove_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let result = self.puts(state_root, (key, None))?;
        let size = result.nodes.values().map(|v| v.len()).sum::<usize>();
        self.metrics
            .smt_remove_bytes
            .with_label_values(&[fn_name])
            .observe(size as f64);
        Ok(result)
    }

    /// Get the value of the key from the tree.
    #[named]
    pub fn get(&self, state_root: H256, key: K) -> Result<Option<V>> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .smt_get_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        Ok(self.get_with_proof(state_root, key)?.0)
    }

    pub fn contains(&self, state_root: H256, key: K) -> Result<bool> {
        self.get(state_root, key).map(|result| result.is_some())
    }

    /// Returns the value and the corresponding merkle proof.
    /// if the value is not applicable, return None and non-inclusion proof.
    #[named]
    pub fn get_with_proof(
        &self,
        state_root: H256,
        key: K,
    ) -> Result<(Option<V>, SparseMerkleProof)> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .smt_get_with_proof_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let tree: JellyfishMerkleTree<K, V, NR> = JellyfishMerkleTree::new(&self.node_reader);
        let (data, proof) = tree.get_with_proof(state_root.into(), key)?;

        let size = data.clone().map(|v| v.raw.len()).unwrap_or(0);
        self.metrics
            .smt_get_with_proof_bytes
            .with_label_values(&[fn_name])
            .observe(size as f64);
        match data {
            Some(b) => Ok((Some(b.origin), proof)),
            None => Ok((None, proof)),
        }
    }

    /// List the (key, value) from the tree.
    #[named]
    pub fn list(
        &self,
        state_root: H256,
        starting_key: Option<K>,
        limit: usize,
    ) -> Result<Vec<(K, V)>> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .smt_list_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let mut iter = self.iter(state_root, starting_key)?;

        let mut data = Vec::new();
        // skip the starting_key if starting_key not NONE
        if Option::is_some(&starting_key) {
            let _item = iter.next();
        }
        for (data_idx, item) in iter.enumerate() {
            if data_idx >= limit {
                break;
            }
            let (k, v) = item?;
            data.push((k, v));
        }
        Ok(data)
    }

    /// Returns the iterator of the tree for scan the tree.
    /// Note: the key in the tree is sorted by the hash of the key, not origin key.
    /// So the iterator will return the key in the hash order, the starting_key is the first key to start scan.
    #[named]
    pub fn iter(&self, state_root: H256, starting_key: Option<K>) -> Result<SMTIterator<K, V, NR>> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .smt_iter_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let iterator = SMTIterator::new(&self.node_reader, state_root, starting_key)?;
        Ok(iterator)
    }

    /// Put kv pairs into tree and generate new state_root.
    #[named]
    pub fn puts<I: Into<UpdateSet<K, V>>>(
        &self,
        state_root: H256,
        update_set: I,
    ) -> Result<TreeChangeSet> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .smt_puts_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let result = self.updates(state_root, update_set)?;
        let size = result.nodes.values().map(|v| v.len()).sum::<usize>();
        self.metrics
            .smt_puts_bytes
            .with_label_values(&[fn_name])
            .observe(size as f64);
        Ok(result)
    }

    fn updates<I: Into<UpdateSet<K, V>>>(
        &self,
        state_root: H256,
        updates: I,
    ) -> Result<TreeChangeSet> {
        let updates: UpdateSet<K, V> = updates.into();
        if updates.is_empty() {
            return Ok(TreeChangeSet {
                state_root,
                nodes: BTreeMap::default(),
            });
        }

        let tree = JellyfishMerkleTree::new(&self.node_reader);
        let (new_state_root, change_set) =
            tree.updates(Some(state_root.into()), updates.into_updates()?)?;

        let mut node_map: BTreeMap<H256, Vec<u8>> = BTreeMap::new();

        for (nk, n) in change_set.node_batch.into_iter() {
            node_map.insert(nk.into(), n.encode()?);
        }

        let new_state_root: H256 = new_state_root.into();

        Ok(TreeChangeSet {
            state_root: new_state_root,
            nodes: node_map,
        })
    }

    /// Dump all (key, value) from the tree
    pub fn dump(&self, state_root: H256) -> Result<Vec<(K, V)>> {
        let iter = self.iter(state_root, None)?;

        let mut data = Vec::new();
        for item in iter {
            let (k, v) = item?;
            data.push((k, v));
        }
        Ok(data)
    }
}

pub struct SMTIterator<'a, K, V, NR>
where
    K: Key,
    V: Value,
    NR: NodeReader,
{
    iter: JellyfishMerkleIterator<'a, K, V, NR>,
}

impl<'a, K, V, NR> SMTIterator<'a, K, V, NR>
where
    K: Key,
    V: Value,
    NR: NodeReader,
{
    pub fn new(reader: &'a NR, state_root: H256, starting_key: Option<K>) -> Result<Self>
    where
        NR: NodeReader,
    {
        let iter = JellyfishMerkleIterator::new(reader, state_root.into(), starting_key)?;
        Ok(SMTIterator { iter })
    }
}

impl<'a, K, V, NR> Iterator for SMTIterator<'a, K, V, NR>
where
    K: Key,
    V: Value,
    NR: NodeReader,
{
    type Item = Result<(K, V)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|result| match result {
            Ok((k, v)) => Ok((k, v.origin)),
            Err(e) => Err(e),
        })
    }
}
