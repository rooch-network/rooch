// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    gas_config::GasConfig, h256, h256::H256, move_types::FunctionId,
    moveos_std::event::TransactionEvent, moveos_std::tx_context::TxContext,
    moveos_std::tx_meta::TxMeta, state::StateChangeSet,
};
use move_core_types::{
    account_address::AccountAddress,
    effects::ChangeSet,
    language_storage::{ModuleId, TypeTag},
    vm_status::KeptVMStatus,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[cfg(any(test, feature = "fuzzing"))]
use crate::move_types::type_tag_prop_strategy;
use crate::moveos_std::event::{Event, EventID};
#[cfg(any(test, feature = "fuzzing"))]
use move_core_types::identifier::Identifier;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;

/// Call a Move script
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScriptCall {
    #[serde(with = "serde_bytes")]
    pub code: Vec<u8>,
    pub ty_args: Vec<TypeTag>,
    //TOOD custom serialize
    pub args: Vec<Vec<u8>>,
}

// Generates random ScriptCall
#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for ScriptCall {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        let ty_args_strategy = prop::collection::vec(type_tag_prop_strategy(), 0..10);

        (
            any::<Vec<u8>>(),
            ty_args_strategy,
            Vec::<Vec<u8>>::arbitrary(),
        )
            .prop_map(|(code, ty_args, args)| ScriptCall {
                code,
                ty_args,
                args,
            })
            .boxed()
    }
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

// Generates random FunctionCall
#[cfg(any(test, feature = "fuzzing"))]
#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for FunctionCall {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        let function_id_strategy = (any::<ModuleId>(), any::<Identifier>())
            .prop_map(|(module_id, identifier)| FunctionId::new(module_id, identifier));
        let ty_args_strategy = prop::collection::vec(type_tag_prop_strategy(), 0..10);

        (
            function_id_strategy,
            ty_args_strategy,
            any::<Vec<Vec<u8>>>(),
        )
            .prop_map(|(function_id, ty_args, args)| FunctionCall {
                function_id,
                ty_args,
                args,
            })
            .boxed()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub enum MoveAction {
    //Execute a Move script
    Script(ScriptCall),
    //Execute a Move function
    Function(FunctionCall),
    //Publish Move modules
    ModuleBundle(Vec<Vec<u8>>),
}

impl MoveAction {
    pub fn action_type(&self) -> u8 {
        match self {
            MoveAction::Script(_) => 0,
            MoveAction::Function(_) => 1,
            MoveAction::ModuleBundle(_) => 2,
        }
    }

    pub fn action_name(&self) -> String {
        match self {
            MoveAction::Script(_) => "Script".to_string(),
            MoveAction::Function(_) => "Function".to_string(),
            MoveAction::ModuleBundle(_) => "ModuleBundle".to_string(),
        }
    }

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

    // Serialize the MoveAction enum into bytes using BCS
    pub fn encode(&self) -> Result<Vec<u8>, anyhow::Error> {
        let encoded_data = bcs::to_bytes(self).expect("Serialization should succeed");
        Ok(encoded_data)
    }
}

impl From<VerifiedMoveAction> for MoveAction {
    fn from(verified_action: VerifiedMoveAction) -> Self {
        match verified_action {
            VerifiedMoveAction::Script { call } => MoveAction::Script(call),
            VerifiedMoveAction::Function { call } => MoveAction::Function(call),
            VerifiedMoveAction::ModuleBundle {
                module_bundle,
                init_function_modules: _init_function_modules,
            } => MoveAction::ModuleBundle(module_bundle),
        }
    }
}

/// The MoveAction after verifier
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VerifiedMoveAction {
    Script {
        call: ScriptCall,
    },
    Function {
        call: FunctionCall,
    },
    ModuleBundle {
        module_bundle: Vec<Vec<u8>>,
        init_function_modules: Vec<ModuleId>,
    },
}

impl Display for VerifiedMoveAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifiedMoveAction::Script { call: _ } => {
                write!(f, "ScriptCall")
            }
            VerifiedMoveAction::Function { call } => {
                write!(f, "FunctionCall(function_id: {})", call.function_id)
            }
            VerifiedMoveAction::ModuleBundle {
                module_bundle,
                init_function_modules,
            } => {
                write!(
                    f,
                    "ModuleBundle(module_bundle: {}, init_function_modules: {})",
                    module_bundle.len(),
                    init_function_modules.len()
                )
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MoveOSTransaction {
    pub ctx: TxContext,
    pub action: MoveAction,
    /// if the pre_execute_functions is not empty, the MoveOS will call the functions before the transaction is executed.
    pub pre_execute_functions: Vec<FunctionCall>,
    /// if the post_execute_functions is not empty, the MoveOS will call the functions after the transaction is executed.
    pub post_execute_functions: Vec<FunctionCall>,
}

impl MoveOSTransaction {
    /// Create a new MoveOS transaction
    /// This function only for test case usage
    pub fn new_for_test(sender: AccountAddress, action: MoveAction) -> Self {
        let sender_and_action = (sender, action);
        let tx_hash = h256::sha3_256_of(bcs::to_bytes(&sender_and_action).unwrap().as_slice());
        //TODO pass the sequence_number
        let ctx = TxContext::new(
            sender_and_action.0,
            0,
            GasConfig::DEFAULT_MAX_GAS_AMOUNT,
            tx_hash,
            1,
        );
        Self::new(ctx, sender_and_action.1)
    }

    pub fn new(mut ctx: TxContext, action: MoveAction) -> Self {
        ctx.add(TxMeta::new_from_move_action(&action))
            .expect("add TxMeta to TxContext should success");
        Self {
            ctx,
            action,
            pre_execute_functions: vec![],
            post_execute_functions: vec![],
        }
    }

    pub fn append_pre_execute_functions(&mut self, functions: Vec<FunctionCall>) {
        self.pre_execute_functions.extend(functions);
    }

    pub fn append_post_execute_functions(&mut self, functions: Vec<FunctionCall>) {
        self.post_execute_functions.extend(functions);
    }
}

#[derive(Debug, Clone)]
pub struct GasStatement {
    pub execution_gas_used: u64,
    pub storage_gas_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedMoveOSTransaction {
    pub ctx: TxContext,
    pub action: VerifiedMoveAction,
    pub pre_execute_functions: Vec<FunctionCall>,
    pub post_execute_functions: Vec<FunctionCall>,
}

/// RawTransactionOutput is the execution result of a MoveOS transaction
//TODO make RawTransactionOutput serializable
#[derive(Debug, Clone)]
pub struct RawTransactionOutput {
    pub status: KeptVMStatus,
    pub changeset: ChangeSet,
    pub state_changeset: StateChangeSet,
    pub events: Vec<TransactionEvent>,
    pub gas_used: u64,
    pub is_upgrade: bool,
    pub gas_statement: GasStatement,
}

/// TransactionOutput is the execution result of a MoveOS transaction, and pack TransactionEvent to Event
//TODO make TransactionOutput serializable
#[derive(Debug, Clone)]
pub struct TransactionOutput {
    pub status: KeptVMStatus,
    pub changeset: ChangeSet,
    pub state_changeset: StateChangeSet,
    pub events: Vec<Event>,
    pub gas_used: u64,
    pub is_upgrade: bool,
}

impl TransactionOutput {
    pub fn new(transaction_output: RawTransactionOutput, event_ids: Vec<EventID>) -> Self {
        debug_assert!(
            transaction_output.events.len() == event_ids.len(),
            "Transaction events len mismatch events len"
        );

        let events = transaction_output
            .events
            .clone()
            .into_iter()
            .enumerate()
            .map(|(index, event)| Event::new_with_event_id(event_ids[index], event))
            .collect::<Vec<_>>();

        TransactionOutput {
            status: transaction_output.status,
            changeset: transaction_output.changeset,
            state_changeset: transaction_output.state_changeset,
            events,
            gas_used: transaction_output.gas_used,
            is_upgrade: transaction_output.is_upgrade,
        }
    }
}

/// `TransactionExecutionInfo` represents the result of executing a transaction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionExecutionInfo {
    /// The hash of this transaction.
    pub tx_hash: H256,

    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    pub state_root: H256,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    pub event_root: H256,

    /// The amount of gas used.
    pub gas_used: u64,

    /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
    /// failures and Move abort's receive more detailed information. But other errors are generally
    /// categorized with no status code or other information
    pub status: KeptVMStatus,
}

impl TransactionExecutionInfo {
    pub fn new(
        tx_hash: H256,
        state_root: H256,
        event_root: H256,
        gas_used: u64,
        status: KeptVMStatus,
    ) -> TransactionExecutionInfo {
        TransactionExecutionInfo {
            tx_hash,
            state_root,
            event_root,
            gas_used,
            status,
        }
    }

    pub fn id(&self) -> H256 {
        h256::sha3_256_of(bcs::to_bytes(self).unwrap().as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::MoveAction;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_move_action_bcs_serde(input in any::<MoveAction>()) {
            let serialized = bcs::to_bytes(&input).unwrap();
            let deserialized: MoveAction = bcs::from_bytes(&serialized).unwrap();
            assert_eq!(input, deserialized);
        }
    }
}
