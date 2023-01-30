// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use jellyfish_merkle::{hash::HashValue, proof::SparseMerkleProof};
use anyhow::Result;

pub mod jellyfish_merkle;

/// Store the tree nodes
pub trait NodeStore{
   
}

/// Store the original values
pub trait ValueStore{
   
}

impl NodeStore for std::collections::HashMap<HashValue, Vec<u8>>{
   
}

impl ValueStore for std::collections::HashMap<HashValue, Vec<u8>>{

}

pub struct SMTree<NS:NodeStore,VS:ValueStore>{
    pub node_store: NS,
    pub value_store: VS,
}

impl<NS:NodeStore,VS:ValueStore> SMTree<NS,VS>{
    pub fn new(node_store: NS, value_store: VS) -> Self{
        SMTree{
            node_store,
            value_store,
        }
    }

    pub fn root(&self) -> HashValue{
        todo!()
    }

    pub fn get<K:AsRef<[u8]>+ ?Sized>(&self, _key: &K) -> Result<Option<Vec<u8>>>{
        todo!()
    }

    pub fn get_with_proof<K:AsRef<[u8]>+ ?Sized>(&self, _key: &K) -> Result<(Option<Vec<u8>>,SparseMerkleProof)>{
        todo!()
    }

    pub fn insert<K:Into<Vec<u8>>, V: Into<Vec<u8>>>(&mut self, _key: K, _value: V) -> Result<()>{
        todo!()
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_smt(){
        let node_store = std::collections::HashMap::new();
        let value_store = std::collections::HashMap::new();
        let mut smt = SMTree::new(node_store, value_store);
        let key = "key";
        let value = "value";
        smt.insert(key.to_string(), value).unwrap();
        smt.get_with_proof(key).unwrap();
    }
}
