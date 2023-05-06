// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use jellyfish_merkle::{
    iterator::JellyfishMerkleIterator,
    node_type::{Node, NodeKey},
    JellyfishMerkleTree, TreeReader,
};
use parking_lot::RwLock;
use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData, sync::Arc,
};

mod jellyfish_merkle;
pub mod smt_object;
#[cfg(test)]
mod tests;
mod update_set;

pub use jellyfish_merkle::{
    hash::{HashValue, SPARSE_MERKLE_PLACEHOLDER_HASH},
    proof::SparseMerkleProof,
};
pub use smt_object::{DecodeToObject, EncodeToObject, Key, SMTObject, Value};
pub use update_set::UpdateSet;

/// Store the tree nodes
pub trait NodeStore {
    fn get(&self, hash: &HashValue) -> Result<Option<Vec<u8>>>;
    fn put(&self, key: HashValue, node: Vec<u8>) -> Result<()>;
    fn write_nodes(&self, nodes: BTreeMap<HashValue, Vec<u8>>) -> Result<()>;
}

impl<K, V, NS> TreeReader<K, V> for NS
where
    NS: NodeStore,
    K: Key,
    V: Value,
{
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node<K, V>>> {
        if node_key == &*SPARSE_MERKLE_PLACEHOLDER_HASH {
            return Ok(Some(Node::new_null()));
        }
        //TODO implement a LRU CachedTreeReader to reduce the decode cost
        self.get(node_key)?
            .map(|v| Node::<K, V>::decode(&v))
            .transpose()
    }
}

#[derive(Default, Clone)]
pub struct InMemoryNodeStore {
    inner: Arc<RwLock<HashMap<HashValue, Vec<u8>>>>,
}

impl From<HashMap<HashValue, Vec<u8>>> for InMemoryNodeStore {
    fn from(map: HashMap<HashValue, Vec<u8>>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(map)),
        }
    }
}

impl NodeStore for InMemoryNodeStore {
    fn get(&self, hash: &HashValue) -> Result<Option<Vec<u8>>> {
        Ok(self.inner.read().get(hash).cloned())
    }

    fn put(&self, key: HashValue, node: Vec<u8>) -> Result<()> {
        self.inner.write().insert(key, node);
        Ok(())
    }

    fn write_nodes(&self, nodes: BTreeMap<HashValue, Vec<u8>>) -> Result<()> {
        self.inner.write().extend(nodes.into_iter());
        Ok(())
    }
}

/// Sparse Merkle Tree
pub struct SMTree<K, V, NS> {
    node_store: NS,
    root_hash: RwLock<HashValue>,
    key: PhantomData<K>,
    value: PhantomData<V>,
}

impl<K, V, NS> SMTree<K, V, NS>
where
    K: Key,
    V: Value,
    NS: NodeStore,
{
    /// Construct a new smt tree from provided `state_root_hash` with underline `node_store`
    pub fn new(node_store: NS, root_hash: Option<HashValue>) -> Self {
        let state_root_hash = root_hash.unwrap_or(*SPARSE_MERKLE_PLACEHOLDER_HASH);
        SMTree {
            node_store,
            root_hash: RwLock::new(state_root_hash),
            key: PhantomData,
            value: PhantomData,
        }
    }

    /// get current root hash
    pub fn root_hash(&self) -> HashValue {
        *self.root_hash.read()
    }

    /// Put a kv pair into tree and generate new state_root.
    /// If need to put many kvs, please use `puts` method.
    pub fn put(&self, key: K, value: V) -> Result<HashValue> {
        self.puts((key, Some(value)))
    }

    /// Remove key_hash's data.
    /// Same as put(K,None)
    pub fn remove(&self, key: K) -> Result<HashValue> {
        self.puts((key, None))
    }

    /// Get the value of the key from the tree.
    pub fn get(&self, key: K) -> Result<Option<V>> {
        Ok(self.get_with_proof(key)?.0)
    }

    pub fn contains(&self, key: K) -> Result<bool> {
        self.get(key).map(|result| result.is_some())
    }

    /// Returns the value and the corresponding merkle proof.
    /// if the value is not applicable, return None and non-inclusion proof.
    pub fn get_with_proof(&self, key: K) -> Result<(Option<V>, SparseMerkleProof)> {
        let cur_root_hash = self.root_hash();

        let tree: JellyfishMerkleTree<K, V, NS> = JellyfishMerkleTree::new(&self.node_store);
        let key = key.into_object();
        let (data, proof) = tree.get_with_proof(cur_root_hash, key)?;
        match data {
            Some(b) => Ok((Some(b.origin), proof)),
            None => Ok((None, proof)),
        }
    }

    /// Returns the iterator of the tree for scan the tree.
    /// Note: the key in the tree is sorted by the hash of the key, not origin key.
    /// So the iterator will return the key in the hash order, the starting_key is the first key to start scan.
    pub fn iter(&self, starting_key: Option<K>) -> Result<SMTIterator<K, V, NS>> {
        let cur_root_hash = self.root_hash();
        let iterator = SMTIterator::new(&self.node_store, cur_root_hash, starting_key)?;
        Ok(iterator)
    }

    /// Put kv pairs into tree and generate new state_root.
    pub fn puts<I: Into<UpdateSet<K, V>>>(&self, update_set: I) -> Result<HashValue> {
        self.updates(update_set)
    }

    fn updates<I: Into<UpdateSet<K, V>>>(&self, updates: I) -> Result<HashValue> {
        let updates: UpdateSet<K, V> = updates.into();
        let cur_root_hash = self.root_hash();
        if updates.is_empty() {
            return Ok(cur_root_hash);
        }

        let tree = JellyfishMerkleTree::new(&self.node_store);
        let (new_state_root, change_set) =
            tree.updates(Some(cur_root_hash), updates.into_updates())?;

        let mut node_map = BTreeMap::new();

        for (nk, n) in change_set.node_batch.into_iter() {
            node_map.insert(nk, n.encode()?);
        }

        self.node_store.write_nodes(node_map)?;
        //TODO handle change_set's stale_node_index
        *self.root_hash.write() = new_state_root;

        Ok(new_state_root)
    }

    pub fn is_genesis(&self) -> bool {
        self.root_hash() == *SPARSE_MERKLE_PLACEHOLDER_HASH
    }
}

pub struct SMTIterator<'a, K, V, R>
where
    R: TreeReader<K, V>,
{
    iter: JellyfishMerkleIterator<'a, K, V, R>,
}

impl<'a, K, V, R> SMTIterator<'a, K, V, R>
where
    K: Key,
    V: Value,
    R: TreeReader<K, V>,
{
    pub fn new(reader: &'a R, root_hash: HashValue, starting_key: Option<K>) -> Result<Self>
    where
        R: TreeReader<K, V>,
    {
        let iter =
            JellyfishMerkleIterator::new(reader, root_hash, starting_key.map(|k| k.into_object()))?;
        Ok(SMTIterator { iter })
    }
}

impl<'a, K, V, R> Iterator for SMTIterator<'a, K, V, R>
where
    K: Key,
    V: Value,
    R: TreeReader<K, V>,
{
    type Item = Result<(K, V)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|result| match result {
            Ok((k, v)) => Ok((k.origin, v.origin)),
            Err(e) => Err(e),
        })
    }
}
