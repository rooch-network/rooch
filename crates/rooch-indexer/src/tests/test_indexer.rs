// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::indexer_reader::IndexerReader;
use crate::store::traits::IndexerStoreTrait;
use crate::types::{
    IndexedEvent, IndexedGlobalState, IndexedTableChangeSet, IndexedTableState, IndexedTransaction,
};
use crate::utils::format_struct_tag;
use crate::IndexerStore;
use anyhow::Result;
use ethers::types::{Bytes, U256};
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::Op;
use move_core_types::language_storage::{ModuleId, StructTag};
use move_core_types::vm_status::KeptVMStatus;
use moveos_types::h256::H256;
use moveos_types::move_types::{random_identity, random_struct_tag, random_type_tag, FunctionId};
use moveos_types::moveos_std::context;
use moveos_types::moveos_std::event::{Event, EventID};
use moveos_types::moveos_std::object::{NamedTableID, ObjectEntity, ObjectID, RawData};
use moveos_types::moveos_std::raw_table::TableInfo;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::state::{
    MoveStructType, SplitStateChangeSet, State, StateChangeSet, TableChange, TableTypeInfo,
};
use moveos_types::transaction::{
    FunctionCall, MoveAction, ScriptCall, TransactionExecutionInfo, VerifiedMoveAction,
    VerifiedMoveOSTransaction,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rooch_config::indexer_config::ROOCH_INDEXER_DB_FILENAME;
use rooch_types::address::{RoochAddress, RoochSupportedAddress};
use rooch_types::framework::coin::CoinInfo;
use rooch_types::framework::gas_coin::GasCoin;
use rooch_types::indexer::event_filter::EventFilter;
use rooch_types::indexer::state::{GlobalStateFilter, TableStateFilter};
use rooch_types::indexer::transaction_filter::TransactionFilter;
use rooch_types::transaction::authenticator::Authenticator;
use rooch_types::transaction::ethereum::EthereumTransaction;
use rooch_types::transaction::rooch::{RoochTransaction, RoochTransactionData};
use rooch_types::transaction::{TransactionSequenceInfo, TypedTransaction};
use std::str::FromStr;

fn random_bytes() -> Vec<u8> {
    H256::random().0.to_vec()
}

pub fn random_string() -> String {
    let mut rng = thread_rng();
    let len = rng.gen_range(1..=100);

    if len == 0 {
        "".to_string()
    } else {
        let mut string = "a".to_string();
        (1..len).for_each(|_| string.push(char::from(rng.sample(Alphanumeric))));
        string
    }
}

fn random_typed_transaction() -> TypedTransaction {
    let mut rng = thread_rng();
    let n = rng.gen_range(1..=100);
    if n % 2 == 0 {
        TypedTransaction::Rooch(random_rooch_transaction())
    } else {
        TypedTransaction::Ethereum(random_ethereum_transaction())
    }
}

fn random_rooch_transaction() -> RoochTransaction {
    let mut rng = thread_rng();
    let sequence_number = rng.gen_range(1..=100);
    let tx_data = RoochTransactionData::new_for_test(
        RoochAddress::random(),
        sequence_number,
        random_move_action(),
    );

    let mut rng = thread_rng();
    let auth_validator_id = rng.gen_range(1..=100);
    let authenticator = Authenticator::new(auth_validator_id, random_bytes());

    RoochTransaction::new(tx_data, authenticator)
}

fn random_ethereum_transaction() -> EthereumTransaction {
    let sender = RoochAddress::random();
    let sequence_number = U256::zero();
    let action = random_move_action();
    let action_bytes =
        Bytes::try_from(bcs::to_bytes(&action).unwrap()).expect("Convert action to bytes failed.");
    EthereumTransaction::new_for_test(sender, sequence_number, action_bytes)
}

fn random_move_action() -> MoveAction {
    let mut rng = thread_rng();
    let n = rng.gen_range(1..=100);
    if n % 3 == 0 {
        random_move_action_script()
    } else if n % 2 == 0 {
        random_move_action_function()
    } else {
        random_move_action_module_bundle()
    }
}

fn random_move_action_script() -> MoveAction {
    MoveAction::Script(random_script_call())
}

fn random_move_action_function() -> MoveAction {
    MoveAction::Function(random_function_call())
}

fn random_move_action_module_bundle() -> MoveAction {
    let mut module_bundle = vec![];
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        module_bundle.push(random_bytes());
    }

    MoveAction::ModuleBundle(module_bundle)
}

fn random_function_call() -> FunctionCall {
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
        args.push(random_bytes());
    }

    FunctionCall {
        function_id,
        ty_args,
        args,
    }
}

fn random_function_calls() -> Vec<FunctionCall> {
    let mut function_calls = vec![];
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        function_calls.push(random_function_call());
    }

    function_calls
}

fn random_script_call() -> ScriptCall {
    let mut ty_args = vec![];
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        ty_args.push(random_type_tag());
    }

    let mut args = vec![];
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        args.push(random_bytes());
    }

    ScriptCall {
        code: random_bytes(),
        ty_args,
        args,
    }
}
fn random_verified_move_action() -> VerifiedMoveAction {
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

fn random_verified_move_action_script() -> VerifiedMoveAction {
    VerifiedMoveAction::Script {
        call: random_script_call(),
    }
}

fn random_verified_move_action_function() -> VerifiedMoveAction {
    VerifiedMoveAction::Function {
        call: random_function_call(),
    }
}

fn random_verified_move_action_module_bundle() -> VerifiedMoveAction {
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

fn random_event_id() -> EventID {
    let event_handle_id = ObjectID::from(AccountAddress::random());
    let mut rng = thread_rng();
    let event_seq = rng.gen_range(1..=100);
    EventID::new(event_handle_id, event_seq)
}

fn random_event() -> Event {
    let mut rng = thread_rng();
    let event_index = rng.gen_range(1..=100);

    Event {
        event_id: random_event_id(),
        event_type: random_struct_tag(),
        event_data: random_bytes(),
        event_index,
    }
}

fn random_table_change() -> TableChange {
    let mut table_change = TableChange::new(random_type_tag());

    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=10) {
        table_change.entries.insert(
            random_bytes(),
            Op::New(State::new(random_bytes(), random_type_tag())),
        );
    }
    table_change
}

fn random_state_change_set() -> StateChangeSet {
    let mut state_change_set = StateChangeSet::default();

    // generate new tables
    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=5) {
        let handle = ObjectID::from(AccountAddress::random());
        state_change_set
            .new_tables
            .insert(handle, TableTypeInfo::new(random_type_tag()));
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
        let handle = NamedTableID::Module(AccountAddress::random()).to_object_id();
        state_change_set
            .changes
            .insert(handle, random_table_change());
    }

    // generate resources change tables
    for _n in 0..rng.gen_range(1..=10) {
        let handle = NamedTableID::Resource(AccountAddress::random()).to_object_id();
        state_change_set
            .changes
            .insert(handle, random_table_change());
    }

    // generate global table
    state_change_set
        .changes
        .insert(context::GLOBAL_OBJECT_STORAGE_HANDLE, random_table_change());

    state_change_set
}

fn random_table_object() -> Result<ObjectEntity<TableInfo>> {
    let table_info = TableInfo::new(AccountAddress::random(), random_type_tag())?;

    Ok(ObjectEntity::new_table_object(
        ObjectID::from(AccountAddress::random()),
        table_info,
    ))
}

#[allow(dead_code)]
fn random_raw_object() -> ObjectEntity<RawData> {
    let raw_data = RawData {
        struct_tag: random_struct_tag(),
        value: random_bytes(),
    };

    ObjectEntity::new_raw_object(ObjectID::from(AccountAddress::random()), raw_data)
}

fn random_update_global_states(states: Vec<IndexedGlobalState>) -> Vec<IndexedGlobalState> {
    states
        .into_iter()
        .map(|item| IndexedGlobalState {
            object_id: item.object_id,
            owner: item.owner,
            flag: item.flag,
            value: random_string(),
            object_type: item.object_type,
            key_type: item.key_type,
            size: item.size + 1,
            tx_order: item.tx_order,
            state_index: item.state_index,
            created_at: item.created_at,
            updated_at: item.updated_at + 1,
        })
        .collect()
}

fn random_new_global_states() -> Result<Vec<IndexedGlobalState>> {
    let mut new_global_states = vec![];

    let mut state_index = 0u64;
    let mut rng = thread_rng();
    for n in 0..rng.gen_range(1..=10) {
        let state = IndexedGlobalState::new_from_table_object(
            random_table_object()?,
            random_string(),
            random_struct_tag().to_canonical_string(),
            random_type_tag().to_canonical_string(),
            n as u64,
            state_index,
        );

        new_global_states.push(state);
        state_index = state_index + 1;
    }

    Ok(new_global_states)
}

fn random_remove_global_states() -> Vec<String> {
    let mut remove_global_states = vec![];

    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=10) {
        let table_handle = ObjectID::from(AccountAddress::random());
        remove_global_states.push(table_handle.to_string());
    }

    remove_global_states
}

fn random_new_table_states() -> Vec<IndexedTableState> {
    let mut table_states = vec![];

    let mut state_index = 0u64;
    let mut rng = thread_rng();
    for n in 0..rng.gen_range(1..=10) {
        let state = IndexedTableState::new(
            ObjectID::from(AccountAddress::random()),
            H256::random().to_string(),
            random_string(),
            random_type_tag(),
            n as u64,
            state_index,
        );
        table_states.push(state);
        state_index = state_index + 1;
    }

    table_states
}

fn random_update_table_states(states: Vec<IndexedTableState>) -> Vec<IndexedTableState> {
    states
        .into_iter()
        .map(|item| IndexedTableState {
            table_handle: item.table_handle,
            key_hex: item.key_hex,
            value: random_string(),
            value_type: random_type_tag(),
            tx_order: item.tx_order,
            state_index: item.state_index,
            created_at: item.created_at,
            updated_at: item.updated_at + 1,
        })
        .collect()
}

fn random_remove_table_states() -> Vec<(String, String)> {
    let mut remove_table_states = vec![];

    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=10) {
        let table_handle = ObjectID::from(AccountAddress::random());
        remove_table_states.push((table_handle.to_string(), random_string()));
    }

    remove_table_states
}

#[test]
fn test_transaction_store() -> Result<()> {
    let tmpdir = moveos_config::temp_dir();
    let indexer_db = tmpdir.path().join(ROOCH_INDEXER_DB_FILENAME);
    if !indexer_db.exists() {
        std::fs::File::create(indexer_db.clone())?;
    }
    let indexer_db_url = indexer_db
        .as_path()
        .to_str()
        .ok_or(anyhow::anyhow!("Invalid mock indexer db dir"))?;
    let indexer_store = IndexerStore::new(indexer_db_url)?;
    indexer_store.create_all_tables_if_not_exists()?;
    let indexer_reader = IndexerReader::new(indexer_db_url)?;

    let random_transaction = random_typed_transaction();

    let tx_order_signature = Authenticator::new(rand::random(), random_bytes());
    let random_sequence_info =
        TransactionSequenceInfo::new(rand::random(), tx_order_signature, H256::random());

    let random_execution_info = TransactionExecutionInfo::new(
        H256::random(),
        H256::random(),
        H256::random(),
        rand::random(),
        KeptVMStatus::Executed,
    );

    let tx_context = TxContext::new_readonly_ctx(AccountAddress::random());
    let move_action = random_verified_move_action();
    let random_moveos_tx = VerifiedMoveOSTransaction {
        ctx: tx_context,
        action: move_action,
        pre_execute_functions: random_function_calls(),
        post_execute_functions: random_function_calls(),
    };

    let indexed_transaction = IndexedTransaction::new(
        random_transaction,
        random_sequence_info,
        random_execution_info,
        random_moveos_tx.clone(),
    )?;
    let transactions = vec![indexed_transaction];
    let _ = indexer_store.persist_transactions(transactions)?;

    let filter = TransactionFilter::Sender(random_moveos_tx.ctx.sender);
    let query_transactions =
        indexer_reader.query_transactions_with_filter(filter, None, 1, true)?;
    assert_eq!(query_transactions.len(), 1);
    Ok(())
}

#[test]
fn test_event_store() -> Result<()> {
    let tmpdir = moveos_config::temp_dir();
    let indexer_db = tmpdir.path().join(ROOCH_INDEXER_DB_FILENAME);
    if !indexer_db.exists() {
        std::fs::File::create(indexer_db.clone())?;
    }
    let indexer_db_url = indexer_db
        .as_path()
        .to_str()
        .ok_or(anyhow::anyhow!("Invalid mock indexer db dir"))?;
    let indexer_store = IndexerStore::new(indexer_db_url)?;
    indexer_store.create_all_tables_if_not_exists()?;
    let indexer_reader = IndexerReader::new(indexer_db_url)?;

    let random_event = random_event();
    let random_transaction = random_typed_transaction();

    let tx_order_signature = Authenticator::new(rand::random(), random_bytes());
    let random_sequence_info =
        TransactionSequenceInfo::new(rand::random(), tx_order_signature, H256::random());

    let tx_context = TxContext::new_readonly_ctx(AccountAddress::random());
    let move_action = random_verified_move_action();
    let random_moveos_tx = VerifiedMoveOSTransaction {
        ctx: tx_context,
        action: move_action,
        pre_execute_functions: random_function_calls(),
        post_execute_functions: random_function_calls(),
    };

    let indexed_event = IndexedEvent::new(
        random_event,
        random_transaction,
        random_sequence_info,
        random_moveos_tx.clone(),
    );
    let events = vec![indexed_event];
    let _ = indexer_store.persist_events(events)?;

    let filter = EventFilter::Sender(random_moveos_tx.ctx.sender);
    let query_events = indexer_reader.query_events_with_filter(filter, None, 1, true)?;
    assert_eq!(query_events.len(), 1);
    Ok(())
}

#[test]
fn test_state_store() -> Result<()> {
    let tmpdir = moveos_config::temp_dir();
    let indexer_db = tmpdir.path().join(ROOCH_INDEXER_DB_FILENAME);
    if !indexer_db.exists() {
        std::fs::File::create(indexer_db.clone())?;
    }
    let indexer_db_url = indexer_db
        .as_path()
        .to_str()
        .ok_or(anyhow::anyhow!("Invalid mock indexer db dir"))?;
    let indexer_store = IndexerStore::new(indexer_db_url)?;
    indexer_store.create_all_tables_if_not_exists()?;
    let indexer_reader = IndexerReader::new(indexer_db_url)?;

    let mut new_global_states = random_new_global_states()?;
    let mut update_global_states = random_update_global_states(new_global_states.clone());
    let remove_global_states = random_remove_global_states();

    let mut new_table_states = random_new_table_states();
    let mut update_table_states = random_update_table_states(new_table_states.clone());
    let remove_table_states = random_remove_table_states();

    //Merge new global states and update global states
    new_global_states.append(&mut update_global_states);
    indexer_store.persist_or_update_global_states(new_global_states)?;
    indexer_store.delete_global_states(remove_global_states)?;

    //Merge new table states and update table states
    new_table_states.append(&mut update_table_states);
    indexer_store.persist_or_update_table_states(new_table_states)?;
    indexer_store.delete_table_states(remove_table_states)?;

    let coin_info_type =
        StructTag::from_str(format_struct_tag(CoinInfo::<GasCoin>::struct_tag()).as_str())?;
    // println!("")
    let filter = GlobalStateFilter::ObjectType(coin_info_type);
    let query_global_states =
        indexer_reader.query_global_states_with_filter(filter, None, 1, true)?;
    assert_eq!(query_global_states.len(), 0);

    let talbe_handle = ObjectID::from_str("0x0")?;
    let filter = TableStateFilter::TableHandle(talbe_handle);
    let query_table_states =
        indexer_reader.query_table_states_with_filter(filter, None, 1, true)?;
    assert_eq!(query_table_states.len(), 0);

    // test state sync
    let state_change_set = random_state_change_set();
    let mut split_state_change_set = SplitStateChangeSet::default();
    for (table_handle, table_info) in state_change_set.new_tables {
        split_state_change_set.add_new_table(table_handle, table_info);
    }
    for (table_handle, table_change) in state_change_set.changes.clone() {
        split_state_change_set.add_table_change(table_handle, table_change);
    }
    for table_handle in state_change_set.removed_tables.clone() {
        split_state_change_set.add_remove_table(table_handle);
    }

    let mut indexed_table_change_sets = vec![];
    for (index, item) in split_state_change_set
        .table_change_sets
        .into_iter()
        .enumerate()
    {
        let table_change_set = IndexedTableChangeSet::new(0, index as u64, item.0, item.1)?;
        indexed_table_change_sets.push(table_change_set);
    }
    indexer_store.persist_table_change_sets(indexed_table_change_sets)?;

    let sync_states = indexer_reader.sync_states(None, None, 2, false)?;
    assert_eq!(sync_states.len(), 2);

    Ok(())
}
