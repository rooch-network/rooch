// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use jellyfish_merkle::{
    node_type::{Node, NodeKey},
    JellyfishMerkleTree, StaleNodeIndex, TreeReader, TreeUpdateBatch,
};
use parking_lot::{Mutex, RwLock};
use std::ops::DerefMut;
use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
};

mod jellyfish_merkle;
pub mod smt_object;

pub use jellyfish_merkle::{
    hash::{HashValue, SPARSE_MERKLE_PLACEHOLDER_HASH},
    proof::SparseMerkleProof,
};
pub use smt_object::{DecodeToObject, EncodeToObject, Key, SMTObject, Value};

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
        self.get(node_key)?
            .map(|v| Node::<K, V>::decode(&v))
            .transpose()
    }
}

#[derive(Default)]
pub struct InMemoryNodeStore {
    inner: RwLock<HashMap<HashValue, Vec<u8>>>,
}

impl From<HashMap<HashValue, Vec<u8>>> for InMemoryNodeStore {
    fn from(map: HashMap<HashValue, Vec<u8>>) -> Self {
        Self {
            inner: RwLock::new(map),
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

#[derive(Clone)]
pub struct StateCache<K, V> {
    root_hash: HashValue,
    change_set_list: Vec<(HashValue, TreeUpdateBatch<K, V>)>,
    split_off_idx: Option<usize>,
}

impl<K, V> StateCache<K, V>
where
    K: Key,
    V: Value,
{
    pub fn new(initial_root: HashValue) -> Self {
        Self {
            root_hash: initial_root,
            change_set_list: Vec::new(),
            split_off_idx: None,
        }
    }

    fn reset(&mut self, root_hash: HashValue) {
        self.root_hash = root_hash;
        self.change_set_list = if let Some(split_idx) = self.split_off_idx {
            self.change_set_list.split_off(split_idx)
        } else {
            Vec::new()
        };
    }

    fn add_changeset(&mut self, root_hash: HashValue, cs: TreeUpdateBatch<K, V>) {
        let mut cur_change_set = TreeUpdateBatch::default();
        let mut cs_num_stale_leaves = cs.num_stale_leaves;
        for stale_node in cs.stale_node_index_batch.iter() {
            match cur_change_set.node_batch.remove(&stale_node.node_key) {
                None => {
                    cur_change_set
                        .stale_node_index_batch
                        .insert(StaleNodeIndex {
                            stale_since_version: root_hash,
                            node_key: stale_node.node_key,
                        });
                }
                Some(n) => {
                    if n.is_leaf() {
                        cur_change_set.num_new_leaves -= 1;
                        cs_num_stale_leaves -= 1;
                    }
                }
            }
        }
        cur_change_set.num_stale_leaves += cs_num_stale_leaves;
        for (nk, n) in cs.node_batch.iter() {
            cur_change_set.node_batch.insert(*nk, n.clone());
            if n.is_leaf() {
                cur_change_set.num_new_leaves += 1;
            }
        }
        self.change_set_list.push((root_hash, cur_change_set));
        self.root_hash = root_hash;
    }
}

struct CachedTreeReader<'a, K, V, NS> {
    store: &'a NS,
    cache: &'a StateCache<K, V>,
}

impl<'a, K, V, NS> TreeReader<K, V> for CachedTreeReader<'a, K, V, NS>
where
    NS: NodeStore,
    K: Key,
    V: Value,
{
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node<K, V>>> {
        if node_key == &*SPARSE_MERKLE_PLACEHOLDER_HASH {
            return Ok(Some(Node::new_null()));
        }
        for change_set in self.cache.change_set_list.iter().rev() {
            if let Some(n) = change_set.1.node_batch.get(node_key).cloned() {
                return Ok(Some(n));
            }
        }
        self.store.get_node_option(node_key)
    }
}

//TODO remove the cache, and support batch update in the jellyfish merkle tree
pub struct SMTree<K, V, NS> {
    node_store: NS,
    storage_root_hash: RwLock<HashValue>,
    updates: RwLock<BTreeMap<K, Option<V>>>,
    cache: Mutex<StateCache<K, V>>,
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
    pub fn new(node_store: NS, state_root_hash: Option<HashValue>) -> Self {
        let state_root_hash = state_root_hash.unwrap_or(*SPARSE_MERKLE_PLACEHOLDER_HASH);
        SMTree {
            node_store,
            storage_root_hash: RwLock::new(state_root_hash),
            updates: RwLock::new(BTreeMap::new()),
            cache: Mutex::new(StateCache::new(state_root_hash)),
            key: PhantomData,
            value: PhantomData,
        }
    }

    /// get current root hash
    /// if any modification is not committed into state tree, the root hash is not changed.
    /// You can use `commit` to make current modification committed into local state tree.
    pub fn root_hash(&self) -> HashValue {
        self.cache.lock().root_hash
    }

    /// put a kv pair into tree.
    /// this will not compute new root hash,
    /// Use `commit` to recompute the root hash.
    pub fn put(&self, key: K, value: V) {
        self.updates.write().insert(key, Some(value));
    }

    /// Remove key_hash's data.
    /// this will not compute new root hash,
    /// Use `commit` to recompute the root hash.
    pub fn remove(&self, key: K) {
        self.updates.write().insert(key, None);
    }

    /// use a key to read a value.
    /// This will also read un-committed modification.
    pub fn get(&self, key: K) -> Result<Option<V>> {
        let updates_guard = self.updates.read();
        if let Some(uncomputed) = updates_guard.get(&key).cloned() {
            return Ok(uncomputed);
        }
        Ok(self.get_with_proof(key)?.0)
    }

    pub fn contains(&self, key: K) -> Result<bool> {
        self.get(key).map(|result| result.is_some())
    }

    /// return value with it proof.
    /// NOTICE: this will only read from state tree.
    /// Any un-committed modification will not visible to the method.
    pub fn get_with_proof(&self, key: K) -> Result<(Option<V>, SparseMerkleProof)> {
        let mut cache_guard = self.cache.lock();
        let cache = cache_guard.deref_mut();
        let cur_root_hash = cache.root_hash;

        let tree: JellyfishMerkleTree<K, V, NS> = JellyfishMerkleTree::new(&self.node_store);
        let key = key.into_object();
        let (data, proof) = tree.get_with_proof(cur_root_hash, key)?;
        match data {
            Some(b) => Ok((Some(b.origin), proof)),
            None => Ok((None, proof)),
        }
    }

    /// Commit current modification into state tree's local cache,
    /// and return new root hash.
    /// NOTICE: this method will not flush the changes into disk.
    /// It just commit the changes into local state-tree, and cache it there.
    pub fn commit(&self) -> Result<HashValue> {
        let mut guard = self.updates.write();
        let updates = guard
            .iter()
            .map(|(k, v)| {
                (
                    k.clone().into_object(),
                    v.clone().map(EncodeToObject::into_object),
                )
            })
            .collect::<Vec<_>>();
        let new_root_hash = self.updates(updates)?;
        guard.clear();
        Ok(new_root_hash)
    }

    /// check if there is data that has not been commit.
    pub fn is_dirty(&self) -> bool {
        self.updates.read().len() > 0
    }

    /// Write state_set to state tree.
    ///TODO
    // pub fn apply(&self, state_set: StateSet) -> Result<()> {
    //     let inner: Vec<(Vec<u8>, Vec<u8>)> = state_set.into();
    //     let updates = inner
    //         .into_iter()
    //         .map(|(k, v)| Ok((K::decode_key(k.as_slice())?, Some(v.into()))))
    //         .collect::<Result<Vec<_>>>();
    //     self.updates(updates?)?;
    //     Ok(())
    // }

    /// commit the state change into underline storage.
    pub fn flush(&self) -> Result<()> {
        let change_set_list = {
            let mut cache_guard = self.cache.lock();
            cache_guard.split_off_idx = Some(cache_guard.change_set_list.len());
            cache_guard.change_set_list.clone()
        };

        //debug!("change_set_list len {}", change_set_list.len());
        // when self::commit call self::updates(&self, updates: Vec<(K, Option<Blob>)>)
        // the param updates is empty cause this situation
        if change_set_list.is_empty() {
            return Ok(());
        }
        let mut root_hash = HashValue::default();
        let mut node_map = BTreeMap::new();
        for (hash, change_sets) in change_set_list.into_iter() {
            for (nk, n) in change_sets.node_batch.into_iter() {
                node_map.insert(nk, n.encode()?);
            }
            root_hash = hash;
        }
        self.node_store.write_nodes(node_map)?;
        // and then advance the storage root hash
        *self.storage_root_hash.write() = root_hash;
        self.cache.lock().reset(root_hash);
        Ok(())
    }

    /// Dump tree to state set.
    // pub fn dump(&self) -> Result<StateSet> {
    //     let cur_root_hash = self.root_hash();
    //     let mut cache_guard = self.cache.lock();
    //     let cache = cache_guard.deref_mut();
    //     let reader = CachedTreeReader {
    //         store: self.storage.as_ref(),
    //         cache,
    //     };
    //     let iterator = JellyfishMerkleIterator::new(&reader, cur_root_hash, HashValue::zero())?;
    //     let mut states = vec![];
    //     for item in iterator {
    //         let item = item?;
    //         states.push((item.0.encode_key()?, item.1.into()));
    //     }
    //     Ok(StateSet::new(states))
    // }

    // pub fn dump_iter(&self) -> Result<JellyfishMerkleIntoIterator<K, StorageTreeReader<K>>> {
    //     let cur_root_hash = self.root_hash();
    //     let cache = {
    //         let cache_guard = self.cache.lock();
    //         cache_guard.clone()
    //     };
    //     let iterator = JellyfishMerkleIntoIterator::new(
    //         StorageTreeReader {
    //             store: self.storage.clone(),
    //             cache,
    //         },
    //         cur_root_hash,
    //         HashValue::zero(),
    //     )?;
    //     Ok(iterator)
    // }

    /// passing None value with a key means delete the key
    fn updates(&self, updates: Vec<(SMTObject<K>, Option<SMTObject<V>>)>) -> Result<HashValue> {
        let cur_root_hash = self.root_hash();
        //TODO should throw a error?
        if updates.is_empty() {
            return Ok(cur_root_hash);
        }
        let mut cache_guard = self.cache.lock();
        let cache = cache_guard.deref_mut();
        let reader = CachedTreeReader {
            store: &self.node_store,
            cache,
        };
        let tree = JellyfishMerkleTree::new(&reader);
        let (new_state_root, change_set) = tree.updates(Some(cur_root_hash), updates)?;
        cache.add_changeset(new_state_root, change_set);
        Ok(new_state_root)
    }

    //
    // /// rollback last write
    // pub fn rollback(&self) {
    //     let mut cache_guard = self.cache.lock();
    //     if let Some(root_hash) = cache_guard.root_hashes.pop() {
    //         let _ = cache_guard.change_sets.pop();
    //     }
    // }
    //
    // /// rollback current state to a history state with the provided `root_hash`
    // pub fn rollback_to(&self, root_hash: HashValue) -> Result<()> {
    //     let mut cache_guard = self.cache.lock();
    //     let mut state_index = None;
    //     for (i, root) in cache_guard.root_hashes.iter().enumerate() {
    //         if root == &root_hash {
    //             state_index = Some(i);
    //         }
    //     }
    //
    //     if let Some(i) = state_index {
    //         cache_guard.truncate(i + 1);
    //     } else if self.storage_root_hash.read().deref() == &root_hash {
    //         cache_guard.clear();
    //     } else {
    //         bail!("the root_hash is not found in write history");
    //     }
    //     Ok(())
    // }

    /// get all changes so far based on initial root_hash.
    /*
    pub fn change_sets(&self) -> (HashValue, TreeUpdateBatch<K>) {
        let cache_guard = self.cache.lock();
        (cache_guard.root_hash, cache_guard.change_set.clone())
    } */

    /// get last changes root_hash
    pub fn last_change_sets(&self) -> Option<(HashValue, TreeUpdateBatch<K, V>)> {
        let cache_guard = self.cache.lock();
        cache_guard.change_set_list.last().cloned()
    }

    // TODO: to keep atomic with other commit.
    // TODO: think about the WriteBatch trait position.
    // pub fn save<T>(&self, batch: &mut T) -> Result<()>
    // where
    //     T: WriteBatch,
    // {
    //     todo!()
    // }

    pub fn is_genesis(&self) -> bool {
        self.root_hash() == *SPARSE_MERKLE_PLACEHOLDER_HASH
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smt() {
        let node_store = InMemoryNodeStore::default();
        let smt = SMTree::new(node_store, None);
        let key = "key";
        let value = "value";
        smt.put(key.to_string(), value.to_string());
        smt.commit().unwrap();
        smt.flush().unwrap();
        let (result, proof) = smt.get_with_proof(key.to_string()).unwrap();
        assert_eq!(result.unwrap(), value.to_string());
        assert!(proof
            .verify(smt.root_hash(), key.to_string(), Some(value.to_string()))
            .is_ok());

        let (result, proof) = smt.get_with_proof("key2".to_string()).unwrap();
        assert_eq!(result, None);
        assert!(proof
            .verify::<String, String>(smt.root_hash(), "key2".to_string(), None)
            .is_ok());
    }
}
