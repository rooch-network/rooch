// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{resolver::{ResourceResolver}, language_storage::{ModuleId, StructTag}, account_address::AccountAddress};

/// StateDB provide state storage and state proof
/// It is a two level SMT storage
pub struct StateDB {

}
/// StateDB query path
/// 1. /account_address/resource_type|module_id
/// 2. /table_handle/key
pub struct AccessPath{

}

pub struct StateProof{

}

pub struct StateProofWithValue{
    pub proof: StateProof,
    pub value: Option<Vec<u8>>
}

impl AccessPath{
    pub fn new_resource_path(_address: &AccountAddress, _tag: &StructTag) -> Self{
        todo!()
    }
    pub fn new_module_path(_module_id: &ModuleId) -> Self{
        todo!()
    }
}

pub trait StateReader{

    fn get(&self, path: &AccessPath) -> Result<Option<Vec<u8>>>;
    fn get_with_proof(&self, path: &AccessPath) -> Result<StateProofWithValue>;
}

pub trait StateWriter{
  
    //TODO define batch struct
    fn write_batch(&self, batch: Vec<(AccessPath, Vec<u8>)>) -> Result<()>;
}

impl StateReader for StateDB{
    
    fn get(&self, _path: &AccessPath) -> Result<Option<Vec<u8>>>{
        todo!()
    }

    fn get_with_proof(&self, _path: &AccessPath) -> Result<StateProofWithValue>{
        todo!()
    }
}

impl StateWriter for StateDB{

    fn write_batch(&self, _batch: Vec<(AccessPath, Vec<u8>)>) -> Result<()>{
        todo!()
    }
}

impl ResourceResolver for StateDB{
    type Error = anyhow::Error;

    fn get_resource(&self, address: &AccountAddress, tag: &StructTag) -> Result<Option<Vec<u8>>, Self::Error> {
        self.get(&AccessPath::new_resource_path(address, tag))
    }
}