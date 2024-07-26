// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use ethers::types::H256;
use rand::{rngs::StdRng, SeedableRng};
use serde::{Deserialize, Serialize};
use smt::{InMemoryNodeStore, SMTree, TreeChangeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Blob {
    data: Vec<u8>,
}

impl From<Vec<u8>> for Blob {
    fn from(data: Vec<u8>) -> Self {
        Blob { data }
    }
}

pub fn gen_kv_from_seed(seed: H256, num_keys: usize) -> Vec<(H256, Option<Blob>)> {
    let mut rng: StdRng = StdRng::from_seed(seed.0);
    let mut kvs = Vec::with_capacity(num_keys);
    for _i in 0..num_keys {
        let key = H256::random_using(&mut rng);
        let value = Blob::from(H256::random_using(&mut rng).0.to_vec());
        kvs.push((key, Some(value)));
    }
    kvs
}

pub fn prepare_change_set(state_root: H256, num_keys: usize) -> Result<(Vec<H256>, TreeChangeSet)> {
    let store = InMemoryNodeStore::default();
    let registry = prometheus::Registry::new();
    let tree: SMTree<H256, Blob, InMemoryNodeStore> = SMTree::new(store, &registry);
    let kvs = gen_kv_from_seed(state_root, num_keys);
    let ks = kvs.iter().map(|(k, _v)| *k).collect();
    let tree_change_set = tree.puts(state_root, kvs)?;
    Ok((ks, tree_change_set))
}
