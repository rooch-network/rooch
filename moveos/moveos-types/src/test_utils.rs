// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    move_types::{random_identity, random_struct_tag, random_type_tag, FunctionId},
    moveos_std::{
        event::{Event, EventID},
        object::{ObjectEntity, ObjectID, ObjectMeta, GENESIS_STATE_ROOT},
        table::TablePlaceholder,
    },
    state::{FieldKey, ObjectChange, ObjectState, StateChangeSet},
    transaction::{FunctionCall, MoveAction, ScriptCall, VerifiedMoveAction},
};
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    effects::Op,
    language_storage::{ModuleId, TypeTag},
};
use rand::{
    distributions::{self, Alphanumeric},
    thread_rng, Rng,
};

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
        bypass_visibility: false,
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

pub fn random_field_change(level: usize) -> (FieldKey, ObjectChange) {
    let (object_id, change) = random_object_change(level + 1);
    (object_id.field_key(), change)
}

pub fn random_raw_object() -> ObjectState {
    let id = ObjectID::from(AccountAddress::random());
    internal_random_object_state(id)
}

pub fn random_raw_object_with_object_id(object_id: ObjectID) -> ObjectState {
    internal_random_object_state(object_id)
}

pub fn random_raw_object_for_child_object(parent_id: ObjectID) -> ObjectState {
    let id = parent_id.child_id(FieldKey::random());
    internal_random_object_state(id)
}

fn internal_random_object_state(id: ObjectID) -> ObjectState {
    let owner = AccountAddress::random();
    let flag = 0u8;
    let state_root = Some(*GENESIS_STATE_ROOT);
    let size = 0;
    let value = random_bytes();
    let created_at = 0;
    let updated_at = 0;
    let value_type = TypeTag::Struct(Box::new(random_struct_tag()));
    let metadata = ObjectMeta::new(
        id, owner, flag, state_root, size, created_at, updated_at, value_type,
    );
    ObjectState::new(metadata, value)
}

pub fn random_object_change(level: usize) -> (ObjectID, ObjectChange) {
    let raw_object = random_raw_object();
    internal_random_object_change(raw_object, level)
}

pub fn random_object_change_with_object_id(
    object_id: ObjectID,
    level: usize,
) -> (ObjectID, ObjectChange) {
    let raw_object = random_raw_object_with_object_id(object_id);
    internal_random_object_change(raw_object, level)
}

pub fn random_object_change_for_child_object(
    parent_id: ObjectID,
    level: usize,
) -> (ObjectID, ObjectChange) {
    let raw_object = random_raw_object_for_child_object(parent_id);
    internal_random_object_change(raw_object, level)
}

fn internal_random_object_change(
    object_state: ObjectState,
    level: usize,
) -> (ObjectID, ObjectChange) {
    let object_id = object_state.id().clone();
    let mut object_change = ObjectChange::new(object_state.metadata, Op::New(object_state.value));

    let mut rng = thread_rng();
    if level <= 2 {
        for _n in 0..rng.gen_range(1..=5) {
            let (key, change) = random_field_change(level + 1);
            object_change.fields.insert(key, change);
        }
    }
    (object_id, object_change)
}

pub fn random_state_change_set() -> StateChangeSet {
    let mut state_change_set = StateChangeSet::default();

    let mut rng = thread_rng();
    let size = rng.gen_range(1..=2);
    state_change_set.root_metadata.size = size;

    // generate changes
    for _n in 0..size {
        let (id, change) = random_object_change(1);
        state_change_set.changes.insert(id.field_key(), change);
    }

    state_change_set
}

pub fn random_state_change_set_for_child_object(parent_id: ObjectID) -> StateChangeSet {
    let mut state_change_set = StateChangeSet::default();

    let mut rng = thread_rng();
    let size = rng.gen_range(1..=20);

    state_change_set.root_metadata.size = size;

    let (_parent_id, parent_object_change) =
        random_object_change_with_object_id(parent_id.clone(), 1);
    state_change_set
        .changes
        .insert(parent_id.field_key(), parent_object_change);

    // generate changes
    for _n in 0..size {
        let (id, change) = random_object_change_for_child_object(parent_id.clone(), 1);
        state_change_set.changes.insert(id.field_key(), change);
    }

    state_change_set
}

pub fn random_table_object() -> Result<ObjectEntity<TablePlaceholder>> {
    Ok(ObjectEntity::new_table_object(
        ObjectID::from(AccountAddress::random()),
        *GENESIS_STATE_ROOT,
        0,
    ))
}
