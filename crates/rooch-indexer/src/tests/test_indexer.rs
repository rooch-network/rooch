// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::indexer_reader::IndexerReader;
use crate::store::traits::IndexerStoreTrait;
use crate::types::{IndexedEvent, IndexedFieldState, IndexedObjectState, IndexedTransaction};
use crate::utils::format_struct_tag;
use crate::IndexerStore;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use move_core_types::vm_status::KeptVMStatus;
use moveos_types::h256::H256;
use moveos_types::move_types::{random_struct_tag, random_type_tag};
use moveos_types::moveos_std::object::{ObjectEntity, ObjectID};
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::state::MoveStructType;
use moveos_types::transaction::{TransactionExecutionInfo, VerifiedMoveOSTransaction};
use rand::{random, thread_rng, Rng};
use rooch_config::indexer_config::ROOCH_INDEXER_DB_DIR;
use rooch_types::framework::coin::CoinInfo;
use rooch_types::framework::gas_coin::GasCoin;
use rooch_types::indexer::event_filter::EventFilter;
use rooch_types::indexer::state::{FieldStateFilter, ObjectStateFilter};
use rooch_types::indexer::transaction_filter::TransactionFilter;
use rooch_types::test_utils::{
    random_event, random_function_calls, random_ledger_transaction, random_string,
    random_table_object, random_verified_move_action,
};
use std::str::FromStr;

fn random_update_object_states(states: Vec<IndexedObjectState>) -> Vec<IndexedObjectState> {
    states
        .into_iter()
        .map(|item| IndexedObjectState {
            object_id: item.object_id,
            owner: item.owner,
            flag: item.flag,
            object_type: item.object_type,
            state_root: item.state_root,
            size: item.size + 1,
            tx_order: item.tx_order,
            state_index: item.state_index,
            created_at: item.created_at,
            updated_at: item.updated_at + 1,
        })
        .collect()
}

fn random_new_object_states() -> Result<Vec<IndexedObjectState>> {
    let mut new_object_states = vec![];

    let mut state_index = 0u64;
    let mut rng = thread_rng();
    for n in 0..rng.gen_range(1..=10) {
        let state = IndexedObjectState::new_from_raw_object(
            random_table_object()?.to_raw(),
            random_struct_tag().to_canonical_string(),
            n as u64,
            state_index,
        );

        new_object_states.push(state);
        state_index = state_index + 1;
    }

    Ok(new_object_states)
}

fn random_remove_object_states() -> Vec<String> {
    let mut remove_object_states = vec![];

    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=10) {
        let object_id = ObjectID::from(AccountAddress::random());
        remove_object_states.push(object_id.to_string());
    }

    remove_object_states
}

fn random_new_field_states() -> Vec<IndexedFieldState> {
    let mut field_states = vec![];

    let mut state_index = 0u64;
    let mut rng = thread_rng();
    for n in 0..rng.gen_range(1..=10) {
        let state = IndexedFieldState::new(
            ObjectID::from(AccountAddress::random()),
            H256::random().to_string(),
            random_type_tag(),
            random_type_tag(),
            n as u64,
            state_index,
        );
        field_states.push(state);
        state_index = state_index + 1;
    }

    field_states
}

fn random_update_field_states(states: Vec<IndexedFieldState>) -> Vec<IndexedFieldState> {
    states
        .into_iter()
        .map(|item| IndexedFieldState {
            object_id: item.object_id,
            key_hex: item.key_hex,
            key_type: random_type_tag(),
            value_type: random_type_tag(),
            tx_order: item.tx_order,
            state_index: item.state_index,
            created_at: item.created_at,
            updated_at: item.updated_at + 1,
        })
        .collect()
}

fn random_remove_field_states() -> Vec<(String, String)> {
    let mut remove_field_states = vec![];

    let mut rng = thread_rng();
    for _n in 0..rng.gen_range(1..=10) {
        let object_id = ObjectID::from(AccountAddress::random());
        remove_field_states.push((object_id.to_string(), random_string()));
    }

    remove_field_states
}

#[test]
fn test_transaction_store() -> Result<()> {
    let tmpdir = moveos_config::temp_dir();
    let indexer_db = tmpdir.path().join(ROOCH_INDEXER_DB_DIR);
    if !indexer_db.exists() {
        std::fs::create_dir_all(indexer_db.clone())?;
    }
    let indexer_store = IndexerStore::new(indexer_db.clone())?;
    indexer_store.create_all_tables_if_not_exists()?;
    let indexer_reader = IndexerReader::new(indexer_db)?;

    let random_transaction = random_ledger_transaction();

    let random_execution_info = TransactionExecutionInfo::new(
        H256::random(),
        H256::random(),
        random(),
        H256::random(),
        rand::random(),
        KeptVMStatus::Executed,
    );

    let tx_context = TxContext::new_readonly_ctx(AccountAddress::random());
    let move_action = random_verified_move_action();
    let random_moveos_tx = VerifiedMoveOSTransaction {
        root: ObjectEntity::genesis_root_object(),
        ctx: tx_context,
        action: move_action,
        pre_execute_functions: random_function_calls(),
        post_execute_functions: random_function_calls(),
    };

    let indexed_transaction = IndexedTransaction::new(
        random_transaction,
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
    let indexer_db = tmpdir.path().join(ROOCH_INDEXER_DB_DIR);
    if !indexer_db.exists() {
        std::fs::create_dir_all(indexer_db.clone())?;
    }
    let indexer_store = IndexerStore::new(indexer_db.clone())?;
    indexer_store.create_all_tables_if_not_exists()?;
    let indexer_reader = IndexerReader::new(indexer_db)?;

    let random_event = random_event();
    let random_transaction = random_ledger_transaction();

    let tx_context = TxContext::new_readonly_ctx(AccountAddress::random());
    let move_action = random_verified_move_action();
    let random_moveos_tx = VerifiedMoveOSTransaction {
        root: ObjectEntity::genesis_root_object(),
        ctx: tx_context,
        action: move_action,
        pre_execute_functions: random_function_calls(),
        post_execute_functions: random_function_calls(),
    };

    let indexed_event =
        IndexedEvent::new(random_event, random_transaction, random_moveos_tx.clone());
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
    let indexer_db = tmpdir.path().join(ROOCH_INDEXER_DB_DIR);
    if !indexer_db.exists() {
        std::fs::create_dir_all(indexer_db.clone())?;
    }
    let indexer_store = IndexerStore::new(indexer_db.clone())?;
    indexer_store.create_all_tables_if_not_exists()?;
    let indexer_reader = IndexerReader::new(indexer_db)?;

    let mut new_object_states = random_new_object_states()?;
    let new_object_ids = new_object_states
        .iter()
        .map(|state| state.object_id.clone())
        .collect::<Vec<ObjectID>>();
    let mut update_object_states = random_update_object_states(new_object_states.clone());
    let remove_object_states = random_remove_object_states();

    let mut new_field_states = random_new_field_states();
    let mut update_field_states = random_update_field_states(new_field_states.clone());
    let remove_field_states = random_remove_field_states();

    //Merge new global states and update global states
    new_object_states.append(&mut update_object_states);
    indexer_store.persist_or_update_object_states(new_object_states.clone())?;
    indexer_store.delete_object_states(remove_object_states)?;

    //Merge new table states and update table states
    new_field_states.append(&mut update_field_states);
    indexer_store.persist_or_update_field_states(new_field_states)?;
    indexer_store.delete_field_states(remove_field_states)?;

    let coin_info_type =
        StructTag::from_str(format_struct_tag(CoinInfo::<GasCoin>::struct_tag()).as_str())?;
    let filter = ObjectStateFilter::ObjectType(coin_info_type);
    let query_object_states =
        indexer_reader.query_object_states_with_filter(filter, None, 1, true)?;
    assert_eq!(query_object_states.len(), 0);

    // test for querying batch objects with filter ObjectStateFilter::ObjectId
    let num_objs = new_object_ids.len();
    let filter = ObjectStateFilter::ObjectId(new_object_ids);
    let query_object_states =
        indexer_reader.query_object_states_with_filter(filter, None, num_objs, true)?;
    assert_eq!(query_object_states.len(), num_objs);

    let talbe_handle = ObjectID::from_str("0x0")?;
    let filter = FieldStateFilter::ObjectId(talbe_handle);
    let query_field_states =
        indexer_reader.query_field_states_with_filter(filter, None, 1, true)?;
    assert_eq!(query_field_states.len(), 0);

    Ok(())
}
