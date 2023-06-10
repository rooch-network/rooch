// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    event::Event,
    h256::{self, H256},
    move_types::FunctionId,
    state::StateChangeSet,
    tx_context::TxContext,
};
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    effects::ChangeSet,
    language_storage::{ModuleId, TypeTag},
    vm_status::KeptVMStatus,
};
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

/// The MoveAction after verifier
#[derive(Clone, Debug)]
pub enum VerifiedMoveAction {
    Script {
        call: ScriptCall,
        resolved_args: Vec<Vec<u8>>,
    },
    Function {
        call: FunctionCall,
        resolved_args: Vec<Vec<u8>>,
    },
    ModuleBundle {
        module_bundle: Vec<Vec<u8>>,
        init_function_modules: Vec<ModuleId>,
    },
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

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MoveOSTransaction {
    pub ctx: TxContext,
    pub action: MoveAction,
}

impl MoveOSTransaction {
    /// Create a new MoveOS transaction
    /// This function only for test case usage
    pub fn new_for_test(sender: AccountAddress, action: MoveAction) -> Self {
        let sender_and_action = (sender, action);
        let tx_hash = h256::sha3_256_of(bcs::to_bytes(&sender_and_action).unwrap().as_slice());
        let ctx = TxContext::new(sender_and_action.0, tx_hash);
        Self {
            ctx,
            action: sender_and_action.1,
        }
    }

    pub fn new(ctx: TxContext, action: MoveAction) -> Self {
        Self { ctx, action }
    }
}

#[derive(Debug, Clone)]
pub struct VerifiedMoveOSTransaction {
    pub ctx: TxContext,
    pub action: VerifiedMoveAction,
}

/// TransactionOutput is the execution result of a MoveOS transaction
//TODO make TransactionOutput serializable
#[derive(Debug, Clone)]
pub struct TransactionOutput {
    pub status: KeptVMStatus,
    pub changeset: ChangeSet,
    pub state_changeset: StateChangeSet,
    pub events: Vec<Event>,
    pub gas_used: u64,
}
