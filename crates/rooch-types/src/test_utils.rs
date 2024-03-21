// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::{RoochAddress, RoochSupportedAddress};
use crate::transaction::authenticator::Authenticator;
use crate::transaction::ethereum::EthereumTransaction;
use crate::transaction::rooch::{RoochTransaction, RoochTransactionData};
use crate::transaction::TypedTransaction;
use anyhow::Result;
use ethers::types::{Bytes, U256};
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::Op;
use move_core_types::language_storage::ModuleId;
use moveos_types::move_types::{random_identity, random_struct_tag, random_type_tag, FunctionId};
use moveos_types::moveos_std::account::Account;
use moveos_types::moveos_std::event::{Event, EventID};
use moveos_types::moveos_std::move_module::ModuleStore;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::moveos_std::object::{ObjectEntity, GENESIS_STATE_ROOT};
use moveos_types::moveos_std::table::TablePlaceholder;
use moveos_types::state::{KeyState, State, StateChangeSet, TableChange};
use moveos_types::transaction::{FunctionCall, MoveAction, ScriptCall, VerifiedMoveAction};
use rand::distributions::Alphanumeric;
use rand::{distributions, thread_rng, Rng};

pub enum MoveActionType {
    //Execute a Move script
    Script,
    //Execute a Move function
    Function,
    //Publish Move modules
    ModuleBundle,
}

impl MoveActionType {
    pub fn action_type(&self) -> u8 {
        match self {
            MoveActionType::Script => 0,
            MoveActionType::Function => 1,
            MoveActionType::ModuleBundle => 2,
        }
    }
}

/// Returns n random bytes with fixed size.
pub fn random_bytes() -> Vec<u8> {
    random_bytes_with_size(32)
}

/// Returns n random bytes with size.
pub fn random_bytes_with_size(len: usize) -> Vec<u8> {
    let rng = thread_rng();
    let range = distributions::Uniform::from(0u8..u8::MAX);
    rng.sample_iter(&range).take(len).collect()
}

pub fn random_string() -> String {
    let mut rng = thread_rng();
    let len = rng.gen_range(1..=500);

    random_string_with_size(len)
}

pub fn random_string_with_size(len: usize) -> String {
    let mut rng = thread_rng();

    if len == 0 {
        "".to_string()
    } else {
        let mut string = "a".to_string();
        (1..len).for_each(|_| string.push(char::from(rng.sample(Alphanumeric))));
        string
    }
}

pub fn random_typed_transaction() -> TypedTransaction {
    let mut rng = thread_rng();
    let n = rng.gen_range(1..=100);
    if n % 2 == 0 {
        TypedTransaction::Rooch(random_rooch_transaction())
    } else {
        TypedTransaction::Ethereum(random_ethereum_transaction())
    }
}

/// Returns rooch typed transaction which move action is move function
pub fn random_typed_transaction_for_rooch_function() -> TypedTransaction {
    TypedTransaction::Rooch(random_rooch_transaction_with_move_action(
        MoveActionType::Function,
    ))
}

pub fn random_rooch_transaction() -> RoochTransaction {
    let move_action_type = random_move_action_type();
    random_rooch_transaction_with_move_action(move_action_type)
}

pub fn random_rooch_transaction_with_move_action(move_action: MoveActionType) -> RoochTransaction {
    let mut rng = thread_rng();
    let sequence_number = rng.gen_range(1..=100);
    let tx_data = RoochTransactionData::new_for_test(
        RoochAddress::random(),
        sequence_number,
        random_move_action_with_action_type(move_action.action_type()),
    );

    let mut rng = thread_rng();
    let auth_validator_id = rng.gen_range(1..=100);
    let authenticator = Authenticator::new(auth_validator_id, random_bytes());

    RoochTransaction::new(tx_data, authenticator)
}

pub fn random_ethereum_transaction() -> EthereumTransaction {
    let sender = RoochAddress::random();
    let sequence_number = U256::zero();
    let move_action_type = random_move_action_type();
    let action = random_move_action_with_action_type(move_action_type.action_type());
    let action_bytes =
        Bytes::try_from(bcs::to_bytes(&action).unwrap()).expect("Convert action to bytes failed.");
    EthereumTransaction::new_for_test(sender, sequence_number, action_bytes)
}

pub fn random_move_action_type() -> MoveActionType {
    let mut rng = thread_rng();
    let n = rng.gen_range(1..=100);
    if n % 5 == 0 {
        MoveActionType::ModuleBundle
    } else if n % 3 == 0 {
        MoveActionType::Script
    } else {
        MoveActionType::Function
    }
}

pub fn random_move_action_with_action_type(action_type: u8) -> MoveAction {
    if MoveActionType::Script.action_type() == action_type {
        random_move_action_script()
    } else if MoveActionType::Function.action_type() == action_type {
        random_move_action_function()
    } else {
        random_move_action_module_bundle()
    }
}

pub fn random_move_action_script() -> MoveAction {
    MoveAction::Script(random_script_call())
}

pub fn random_move_action_function() -> MoveAction {
    MoveAction::Function(random_function_call())
}

pub fn random_move_action_module_bundle() -> MoveAction {
    let mut module_bundle = vec![];
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        let bytes_len = rng.gen_range(2000..=20000);
        module_bundle.push(random_bytes_with_size(bytes_len));
    }

    MoveAction::ModuleBundle(module_bundle)
}

pub fn random_function_call() -> FunctionCall {
    let function_id = FunctionId::new(
        ModuleId::new(AccountAddress::random(), random_identity()),
        random_identity(),
    );

    let mut ty_args = vec![];
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        ty_args.push(random_type_tag());
    }

    let mut args = vec![];
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        let bytes_len = rng.gen_range(20..=100);
        args.push(random_bytes_with_size(bytes_len));
    }

    FunctionCall {
        function_id,
        ty_args,
        args,
    }
}

pub fn random_function_calls() -> Vec<FunctionCall> {
    let mut function_calls = vec![];
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        function_calls.push(random_function_call());
    }

    function_calls
}

pub fn random_script_call() -> ScriptCall {
    let mut ty_args = vec![];
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        ty_args.push(random_type_tag());
    }

    let mut args = vec![];
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        let bytes_len = rng.gen_range(20..=200);
        args.push(random_bytes_with_size(bytes_len));
    }

    ScriptCall {
        code: random_bytes(),
        ty_args,
        args,
    }
}
pub fn random_verified_move_action() -> VerifiedMoveAction {
    let mut rng = thread_rng();
    let n = rng.gen_range(1..=100);
    if n % 3 == 0 {
        random_verified_move_action_script()
    } else if n % 2 == 0 {
        random_verified_move_action_function()
    } else {
        random_verified_move_action_module_bundle()
    }
}

pub fn random_verified_move_action_script() -> VerifiedMoveAction {
    VerifiedMoveAction::Script {
        call: random_script_call(),
    }
}

pub fn random_verified_move_action_function() -> VerifiedMoveAction {
    VerifiedMoveAction::Function {
        call: random_function_call(),
    }
}

pub fn random_verified_move_action_module_bundle() -> VerifiedMoveAction {
    let mut module_bundle = vec![];
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        module_bundle.push(random_bytes());
    }

    let mut init_function_modules = vec![];
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        init_function_modules.push(ModuleId::new(AccountAddress::random(), random_identity()));
    }

    VerifiedMoveAction::ModuleBundle {
        module_bundle,
        init_function_modules,
    }
}

pub fn random_event_id() -> EventID {
    let event_handle_id = ObjectID::from(AccountAddress::random());
    let mut rng = thread_rng();
    let event_seq = rng.gen_range(1..=100);
    EventID::new(event_handle_id, event_seq)
}

pub fn random_event() -> Event {
    let mut rng = thread_rng();
    let event_index = rng.gen_range(1..=100);

    Event {
        event_id: random_event_id(),
        event_type: random_struct_tag(),
        event_data: random_bytes(),
        event_index,
    }
}

pub fn random_table_change() -> TableChange {
    let mut table_change = TableChange::default();

    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=10) {
        table_change.entries.insert(
            KeyState::new(random_bytes(), random_type_tag()),
            Op::New(State::new(random_bytes(), random_type_tag())),
        );
    }
    table_change
}

pub fn random_state_change_set() -> StateChangeSet {
    let mut state_change_set = StateChangeSet::default();

    // generate new tables
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        let handle = ObjectID::from(AccountAddress::random());
        state_change_set.new_tables.insert(handle);
    }

    // generate remove tables
    for _n in 0..rng.gen_range(1..=5) {
        let handle = ObjectID::from(AccountAddress::random());
        state_change_set.removed_tables.insert(handle);
    }

    // generate change tables
    for _n in 0..rng.gen_range(1..=5) {
        let handle = ObjectID::from(AccountAddress::random());
        state_change_set
            .changes
            .insert(handle, random_table_change());
    }

    // generate modules change tables
    for _n in 0..rng.gen_range(1..=5) {
        let module_object_id = ModuleStore::module_store_id();
        state_change_set
            .changes
            .insert(module_object_id, random_table_change());
    }

    // generate resources change tables
    for _n in 0..rng.gen_range(1..=10) {
        let account_object_id = Account::account_object_id(AccountAddress::random());
        state_change_set
            .changes
            .insert(account_object_id, random_table_change());
    }

    // generate global table
    state_change_set
        .changes
        .insert(ObjectID::root(), random_table_change());

    state_change_set
}

pub fn random_table_object() -> Result<ObjectEntity<TablePlaceholder>> {
    Ok(ObjectEntity::new_table_object(
        ObjectID::from(AccountAddress::random()),
        *GENESIS_STATE_ROOT,
        0,
    ))
}
