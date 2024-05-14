// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    moveos_std::{
        object::{self, ObjectID},
        tx_context::TxContext,
    },
    state::{MoveStructState, MoveStructType},
    transaction::FunctionCall,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("chain_id");

#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
pub struct ChainID {
    pub id: u64,
}

impl ChainID {
    pub fn new(id: u64) -> Self {
        ChainID { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn chain_id_object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }
}

impl std::fmt::Display for ChainID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl FromStr for ChainID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id: u64 = s.parse()?;
        Ok(Self::new(id))
    }
}

impl From<u64> for ChainID {
    fn from(id: u64) -> Self {
        Self::new(id)
    }
}

impl MoveStructType for ChainID {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("ChainID");
}

impl MoveStructState for ChainID {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U64,
        ])
    }
}

/// Rust bindings for RoochFramework chain_id module
pub struct ChainIDModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> ChainIDModule<'a> {
    pub const CHAIN_ID_FUNCTION_NAME: &'static IdentStr = ident_str!("chain_id");

    pub fn chain_id(&self) -> Result<u64> {
        let call = FunctionCall::new(
            Self::function_id(Self::CHAIN_ID_FUNCTION_NAME),
            vec![],
            vec![],
        );
        let ctx = TxContext::random_for_testing_only();
        Ok(self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<u64>(&value.value).expect("should be a valid u64")
            })?)
    }
}

impl<'a> ModuleBinding<'a> for ChainIDModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_chain_id_object_id() {
        let chain_id_object_id = ChainID::chain_id_object_id();
        //println!("{:?}", chain_id_object_id);
        assert_eq!(
            chain_id_object_id,
            ObjectID::from_str(
                "0x687e4c198ba77fd246ed82ea1fc88bd165a44ad8614f62f9c33e4e658152d7b1"
            )
            .unwrap()
        );
    }
}
