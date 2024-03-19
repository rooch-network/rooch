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
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use move_core_types::vm_status::KeptVMStatus;
use moveos_types::h256::H256;
use moveos_types::move_types::{random_struct_tag, random_type_tag};
use moveos_types::moveos_std::object_id::ObjectID;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::state::{MoveStructType, SplitStateChangeSet};
use moveos_types::transaction::{TransactionExecutionInfo, VerifiedMoveOSTransaction};
use rand::{random, thread_rng, Rng};
use rooch_config::indexer_config::ROOCH_INDEXER_DB_DIR;
use rooch_types::framework::coin::CoinInfo;
use rooch_types::framework::gas_coin::GasCoin;
use rooch_types::indexer::event_filter::EventFilter;
use rooch_types::indexer::state::{GlobalStateFilter, TableStateFilter};
use rooch_types::indexer::transaction_filter::TransactionFilter;
use rooch_types::test_utils::{
    random_bytes, random_event, random_function_calls, random_state_change_set, random_string,
    random_table_object, random_typed_transaction, random_verified_move_action,
};
use rooch_types::transaction::authenticator::Authenticator;
use rooch_types::transaction::TransactionSequenceInfo;
use std::str::FromStr;

fn random_update_global_states(states: Vec<IndexedGlobalState>) -> Vec<IndexedGlobalState> {
    states
        .into_iter()
        .map(|item| IndexedGlobalState {
            object_id: item.object_id,
            owner: item.owner,
            flag: item.flag,
            value: random_string(),
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

fn random_new_global_states() -> Result<Vec<IndexedGlobalState>> {
    let mut new_global_states = vec![];

    let mut state_index = 0u64;
    let mut rng = thread_rng();
    for n in 0..rng.gen_range(1..=10) {
        let state = IndexedGlobalState::new_from_raw_object(
            random_table_object()?.to_raw(),
            random_string(),
            random_struct_tag().to_canonical_string(),
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
            random_string(),
            random_type_tag(),
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
            key_str: random_string(),
            value: random_string(),
            key_type: random_type_tag(),
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
    let indexer_db = tmpdir.path().join(ROOCH_INDEXER_DB_DIR);
    if !indexer_db.exists() {
        std::fs::create_dir_all(indexer_db.clone())?;
    }
    let indexer_store = IndexerStore::new(indexer_db.clone())?;
    indexer_store.create_all_tables_if_not_exists()?;
    let indexer_reader = IndexerReader::new(indexer_db)?;

    let random_transaction = random_typed_transaction();

    let tx_order_signature = Authenticator::new(rand::random(), random_bytes());
    let random_sequence_info =
        TransactionSequenceInfo::new(rand::random(), tx_order_signature, H256::random());

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
    let indexer_db = tmpdir.path().join(ROOCH_INDEXER_DB_DIR);
    if !indexer_db.exists() {
        std::fs::create_dir_all(indexer_db.clone())?;
    }
    let indexer_store = IndexerStore::new(indexer_db.clone())?;
    indexer_store.create_all_tables_if_not_exists()?;
    let indexer_reader = IndexerReader::new(indexer_db)?;

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
    let indexer_db = tmpdir.path().join(ROOCH_INDEXER_DB_DIR);
    if !indexer_db.exists() {
        std::fs::create_dir_all(indexer_db.clone())?;
    }
    let indexer_store = IndexerStore::new(indexer_db.clone())?;
    indexer_store.create_all_tables_if_not_exists()?;
    let indexer_reader = IndexerReader::new(indexer_db)?;

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
    for table_handle in state_change_set.new_tables {
        split_state_change_set.add_new_table(table_handle);
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
