// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::h256::H256;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
};
use serde::{Deserialize, Serialize};

use crate::h256;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Script {
    #[serde(with = "serde_bytes")]
    pub code: Vec<u8>,
    pub ty_args: Vec<TypeTag>,
    //TOOD custom serialize
    pub args: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub module: ModuleId,
    pub function: Identifier,
    pub ty_args: Vec<TypeTag>,
    //TOOD custom serialize
    pub args: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MoveAction {
    //Execute a Move script
    Script(Script),
    //Execute a Move function
    Function(Function),
    //Publish Move modules
    ModuleBundle(Vec<Vec<u8>>),
}

impl MoveAction {
    pub fn new_module_bundle(modules: Vec<Vec<u8>>) -> Self {
        Self::ModuleBundle(modules)
    }
    pub fn new_function(
        module: ModuleId,
        function: Identifier,
        ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    ) -> Self {
        Self::Function(Function {
            module,
            function,
            ty_args,
            args,
        })
    }
    pub fn new_script(code: Vec<u8>, ty_args: Vec<TypeTag>, args: Vec<Vec<u8>>) -> Self {
        Self::Script(Script {
            code,
            ty_args,
            args,
        })
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ViewPayload {
    pub function: Function,
}
