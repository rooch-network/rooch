// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    h256::{self, H256},
    move_types::FunctionId,
};
use anyhow::Result;
use move_core_types::{account_address::AccountAddress, language_storage::TypeTag};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Call a Move script
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScriptCall {
    #[serde(with = "serde_bytes")]
    pub code: Vec<u8>,
    pub ty_args: Vec<TypeTag>,
    //TOOD custom serialize
    pub args: Vec<Vec<u8>>,
}

/// Call a Move function
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    pub function_id: FunctionId,
    pub ty_args: Vec<TypeTag>,
    //TOOD custom serialize
    pub args: Vec<Vec<u8>>,
}

impl FunctionCall {
    pub fn new(function_id: FunctionId, ty_args: Vec<TypeTag>, args: Vec<Vec<u8>>) -> Self {
        Self {
            function_id,
            ty_args,
            args,
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MoveAction {
    //Execute a Move script
    Script(ScriptCall),
    //Execute a Move function
    Function(FunctionCall),
    //Publish Move modules
    ModuleBundle(Vec<Vec<u8>>),
}

impl MoveAction {
    pub fn new_module_bundle(modules: Vec<Vec<u8>>) -> Self {
        Self::ModuleBundle(modules)
    }
    pub fn new_function_call(
        function_id: FunctionId,
        ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    ) -> Self {
        Self::Function(FunctionCall {
            function_id,
            ty_args,
            args,
        })
    }
    pub fn new_script_call(code: Vec<u8>, ty_args: Vec<TypeTag>, args: Vec<Vec<u8>>) -> Self {
        Self::Script(ScriptCall {
            code,
            ty_args,
            args,
        })
    }
}

pub trait AuthenticatableTransaction {
    type AuthenticatorInfo: Serialize;
    type AuthenticatorResult: DeserializeOwned;

    fn tx_hash(&self) -> H256;
    fn authenticator_info(&self) -> Self::AuthenticatorInfo;
    fn construct_moveos_transaction(
        &self,
        result: Self::AuthenticatorResult,
    ) -> Result<MoveOSTransaction>;
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct MoveOSTransaction {
    pub sender: AccountAddress,
    pub action: MoveAction,
    pub tx_hash: H256,
}

impl MoveOSTransaction {
    /// Create a new MoveOS transaction
    /// This function only for test case usage
    pub fn new_for_test(sender: AccountAddress, action: MoveAction) -> Self {
        let sender_and_action = (sender, action);
        let tx_hash = h256::sha3_256_of(bcs::to_bytes(&sender_and_action).unwrap().as_slice());
        Self {
            sender: sender_and_action.0,
            action: sender_and_action.1,
            tx_hash,
        }
    }

    pub fn new(sender: AccountAddress, action: MoveAction, tx_hash: H256) -> Self {
        Self {
            sender,
            action,
            tx_hash,
        }
    }
}
