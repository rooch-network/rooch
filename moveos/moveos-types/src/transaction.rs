// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    h256::{self, H256},
    move_types::FunctionId,
    moveos_std::{
        event::TransactionEvent, gas_schedule::GasScheduleConfig, object::ObjectMeta,
        tx_context::TxContext, tx_meta::TxMeta,
    },
    state::StateChangeSet,
};
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, TypeTag},
    vm_status::KeptVMStatus,
};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::fmt::{self, Display, Formatter};

#[cfg(any(test, feature = "fuzzing"))]
use crate::move_types::type_tag_prop_strategy;
use crate::moveos_std::event::Event;
use crate::test_utils::random_state_change_set;
#[cfg(any(test, feature = "fuzzing"))]
use move_core_types::identifier::Identifier;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use rand::random;
use schemars::JsonSchema;

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
            VerifiedMoveAction::Function {
                call,
                bypass_visibility: _,
            } => MoveAction::Function(call),
            VerifiedMoveAction::ModuleBundle {
                module_bundle,
                init_function_modules: _init_function_modules,
            } => MoveAction::ModuleBundle(module_bundle),
        }
    }
}

impl From<FunctionCall> for MoveAction {
    fn from(call: FunctionCall) -> Self {
        MoveAction::Function(call)
    }
}

impl From<ScriptCall> for MoveAction {
    fn from(call: ScriptCall) -> Self {
        MoveAction::Script(call)
    }
}

impl Display for MoveAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MoveAction::Script(script) => {
                let code_hex = hex::encode(script.code.clone());
                let mut arg_list = vec![];
                for arg in script.args.iter() {
                    arg_list.push(format!("0x{:}", hex::encode(arg)));
                }
                write!(
                    f,
                    "MoveAction::ScriptCall( code: 0x{:?},  type_args: {:?}, args: {:?})",
                    code_hex, script.ty_args, arg_list
                )
            }
            MoveAction::Function(function) => {
                let mut arg_list = vec![];
                for arg in function.args.iter() {
                    arg_list.push(format!("0x{:}", hex::encode(arg)));
                }
                write!(
                    f,
                    "MoveAction::FunctionCall( function_id: {},  type_args: {:?}, args: {:?})",
                    function.function_id, function.ty_args, arg_list
                )
            }
            MoveAction::ModuleBundle(module_bundle) => {
                let mut module_list = vec![];
                for arg in module_bundle.iter() {
                    module_list.push(format!("0x{:}", hex::encode(arg)));
                }
                write!(f, "MoveAction::ModuleBundle( {:?} )", module_list)
            }
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
        bypass_visibility: bool,
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
            VerifiedMoveAction::Function {
                call,
                bypass_visibility,
            } => {
                write!(
                    f,
                    "FunctionCall(function_id: {}, bypass_visibility:{})",
                    call.function_id, bypass_visibility
                )
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

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct MoveOSTransaction {
    pub root: ObjectMeta,
    pub ctx: TxContext,
    pub action: MoveAction,
}

impl MoveOSTransaction {
    /// Create a new MoveOS transaction
    /// This function only for test case usage
    pub fn new_for_test(root: ObjectMeta, sender: AccountAddress, action: MoveAction) -> Self {
        let sender_and_action = (sender, action);
        let tx_hash = h256::sha3_256_of(bcs::to_bytes(&sender_and_action).unwrap().as_slice());
        //TODO pass the sequence_number
        let ctx = TxContext::new(
            sender_and_action.0,
            0,
            GasScheduleConfig::INITIAL_MAX_GAS_AMOUNT,
            tx_hash,
            1,
        );
        Self::new(root, ctx, sender_and_action.1)
    }

    pub fn new(root: ObjectMeta, mut ctx: TxContext, action: MoveAction) -> Self {
        ctx.add(TxMeta::new_from_move_action(&action))
            .expect("add TxMeta to TxContext should success");
        Self { root, ctx, action }
    }
}

/// Custom deserialization logic for MoveOSTransaction
/// `MoveOSTransaction` has been changed from a struct with 5 fields to a struct with 3 fields.
/// The old one was defined:
/// ```rust
/// pub struct MoveOSTransaction {
///     pub root: ObjectMeta,
///     pub ctx: TxContext,
///     pub action: MoveAction,
///     pub pre_execute_functions: Vec<FunctionCall>,
///     pub post_execute_functions: Vec<FunctionCall>,
/// }
/// ```
/// Some old transactions are still stored in the database or genesis file,
/// so we need to support deserializing the old format.
impl<'de> Deserialize<'de> for MoveOSTransaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MoveOSTransactionVisitor;

        impl<'de> Visitor<'de> for MoveOSTransactionVisitor {
            type Value = MoveOSTransaction;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct MoveOSTransaction with 5 or 3 fields")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let root = seq
                    .next_element::<ObjectMeta>()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let ctx = seq
                    .next_element::<TxContext>()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let action = seq
                    .next_element::<MoveAction>()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let _pre_execution_functions: Vec<FunctionCall> =
                    match seq.next_element::<Vec<FunctionCall>>() {
                        Ok(Some(pre_execution_functions)) => pre_execution_functions,
                        Ok(None) => vec![],
                        Err(_e) => {
                            vec![]
                        }
                    };
                let _post_execution_functions: Vec<FunctionCall> =
                    match seq.next_element::<Vec<FunctionCall>>() {
                        Ok(Some(post_execution_functions)) => post_execution_functions,
                        Ok(None) => vec![],
                        Err(_e) => {
                            vec![]
                        }
                    };
                Ok(MoveOSTransaction { root, ctx, action })
            }
        }

        deserializer.deserialize_struct(
            "MoveOSTransaction",
            &[
                "root",
                "ctx",
                "action",
                "pre_execution_functions",
                "post_execution_functions",
            ],
            MoveOSTransactionVisitor,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedMoveOSTransaction {
    pub root: ObjectMeta,
    pub ctx: TxContext,
    pub action: VerifiedMoveAction,
}

impl VerifiedMoveOSTransaction {
    pub fn new(root: ObjectMeta, ctx: TxContext, action: VerifiedMoveAction) -> Self {
        Self { root, ctx, action }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VMErrorInfo {
    pub error_message: String,
    pub execution_state: Vec<String>,
}

impl Default for VMErrorInfo {
    fn default() -> Self {
        Self {
            error_message: "".to_string(),
            execution_state: vec![],
        }
    }
}

/// RawTransactionOutput is the execution result of a MoveOS transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawTransactionOutput {
    pub status: KeptVMStatus,
    //The changeset in RawTransactionOutput is not the same as the changeset in TransactionOutput
    //Because the changeset do not apply to the state tree, so it's StateRoot not updated
    pub changeset: StateChangeSet,
    pub events: Vec<TransactionEvent>,
    pub gas_used: u64,
    pub is_upgrade: bool,
    pub is_gas_upgrade: bool,
}

/// TransactionOutput is the execution result of a MoveOS transaction, and pack TransactionEvent to Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOutput {
    pub status: KeptVMStatus,
    pub changeset: StateChangeSet,
    pub events: Vec<Event>,
    pub gas_used: u64,
    pub is_upgrade: bool,
}

impl TransactionOutput {
    pub fn new(
        status: KeptVMStatus,
        changeset: StateChangeSet,
        events: Vec<Event>,
        gas_used: u64,
        is_upgrade: bool,
    ) -> Self {
        TransactionOutput {
            status,
            changeset,
            events,
            gas_used,
            is_upgrade,
        }
    }

    pub fn random() -> Self {
        TransactionOutput::new(
            KeptVMStatus::Executed,
            random_state_change_set(),
            vec![],
            0,
            false,
        )
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

    /// The root Object count of Sparse Merkle Tree describing the world state at the end of this transaction.
    pub size: u64,

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
        size: u64,
        event_root: H256,
        gas_used: u64,
        status: KeptVMStatus,
    ) -> TransactionExecutionInfo {
        TransactionExecutionInfo {
            tx_hash,
            state_root,
            size,
            event_root,
            gas_used,
            status,
        }
    }

    pub fn id(&self) -> H256 {
        h256::sha3_256_of(bcs::to_bytes(self).unwrap().as_slice())
    }

    pub fn root_metadata(&self) -> ObjectMeta {
        ObjectMeta::root_metadata(self.state_root, self.size)
    }

    pub fn random() -> Self {
        TransactionExecutionInfo::new(
            H256::random(),
            H256::random(),
            random(),
            H256::random(),
            rand::random(),
            KeptVMStatus::Executed,
        )
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
