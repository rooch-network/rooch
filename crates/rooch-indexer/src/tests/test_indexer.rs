// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::indexer_reader::IndexerReader;
use crate::store::traits::IndexerStoreTrait;
use crate::IndexerStore;
use anyhow::Result;
use metrics::RegistryService;
use move_core_types::account_address::AccountAddress;
use move_core_types::vm_status::KeptVMStatus;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::{ObjectEntity, ObjectID, ObjectMeta};
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::state::MoveStructType;
use moveos_types::transaction::{TransactionExecutionInfo, VerifiedMoveOSTransaction};
use rand::{random, thread_rng, Rng};
use rooch_config::store_config::DEFAULT_DB_INDEXER_SUBDIR;
use rooch_types::framework::coin_store::CoinStore;
use rooch_types::framework::gas_coin::GasCoin;
use rooch_types::indexer::event::{EventFilter, IndexerEvent};
use rooch_types::indexer::state::{IndexerObjectState, ObjectStateFilter};
use rooch_types::indexer::transaction::{IndexerTransaction, TransactionFilter};
use rooch_types::test_utils::{
    random_event, random_ledger_transaction, random_table_object, random_verified_move_action,
};

fn random_update_object_states(states: Vec<IndexerObjectState>) -> Vec<IndexerObjectState> {
    states
        .into_iter()
        .map(|item| {
            let mut metadata = item.metadata;
            metadata.size += 1;
            metadata.updated_at += 1;

            IndexerObjectState {
                metadata,
                tx_order: item.tx_order,
                state_index: item.state_index,
            }
        })
        .collect()
}

fn random_new_object_states() -> Result<Vec<IndexerObjectState>> {
    let mut new_object_states = vec![];

    let mut rng = thread_rng();
    for (state_index, n) in (0..rng.gen_range(1..=10)).enumerate() {
        let state = IndexerObjectState::new(
            random_table_object()?.into_state().metadata,
            n as u64,
            state_index as u64,
        );

        new_object_states.push(state);
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

#[test]
fn test_transaction_store() -> Result<()> {
    let registry_service = RegistryService::default();
    let tmpdir = moveos_config::temp_dir();
    let indexer_db = tmpdir.path().join(DEFAULT_DB_INDEXER_SUBDIR);
    let indexer_store =
        IndexerStore::new(indexer_db.clone(), &registry_service.default_registry())?;
    let indexer_reader = IndexerReader::new(indexer_db, &registry_service.default_registry())?;

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
        root: ObjectMeta::genesis_root(),
        ctx: tx_context,
        action: move_action,
    };

    let indexer_transaction = IndexerTransaction::new(
        random_transaction,
        random_execution_info,
        random_moveos_tx.action.into(),
        random_moveos_tx.ctx.clone(),
    )?;
    let transactions = vec![indexer_transaction];
    indexer_store.persist_transactions(transactions)?;

    let filter = TransactionFilter::Sender(random_moveos_tx.ctx.sender.into());
    let query_transactions =
        indexer_reader.query_transactions_with_filter(filter, None, 1, true)?;
    assert_eq!(query_transactions.len(), 1);
    Ok(())
}

#[test]
fn test_event_store() -> Result<()> {
    let registry_service = RegistryService::default();
    let tmpdir = moveos_config::temp_dir();
    let indexer_db = tmpdir.path().join(DEFAULT_DB_INDEXER_SUBDIR);
    let indexer_store =
        IndexerStore::new(indexer_db.clone(), &registry_service.default_registry())?;
    let indexer_reader = IndexerReader::new(indexer_db, &registry_service.default_registry())?;

    let random_event = random_event();
    let random_transaction = random_ledger_transaction();

    let tx_context = TxContext::new_readonly_ctx(AccountAddress::random());
    let move_action = random_verified_move_action();
    let random_moveos_tx = VerifiedMoveOSTransaction {
        root: ObjectMeta::genesis_root(),
        ctx: tx_context,
        action: move_action,
    };

    let indexer_event = IndexerEvent::new(
        random_event,
        random_transaction,
        random_moveos_tx.ctx.clone(),
    );
    let events = vec![indexer_event];
    indexer_store.persist_events(events)?;

    let filter = EventFilter::Sender(random_moveos_tx.ctx.sender.into());
    let query_events = indexer_reader.query_events_with_filter(filter, None, 1, true)?;
    assert_eq!(query_events.len(), 1);
    Ok(())
}

#[test]
fn test_state_store() -> Result<()> {
    let registry_service = RegistryService::default();
    let tmpdir = moveos_config::temp_dir();
    let indexer_db = tmpdir.path().join(DEFAULT_DB_INDEXER_SUBDIR);
    let indexer_store =
        IndexerStore::new(indexer_db.clone(), &registry_service.default_registry())?;
    let indexer_reader = IndexerReader::new(indexer_db, &registry_service.default_registry())?;

    let mut new_object_states = random_new_object_states()?;
    let new_object_ids = new_object_states
        .iter()
        .map(|state| state.metadata.id.clone())
        .collect::<Vec<ObjectID>>();
    let mut update_object_states = random_update_object_states(new_object_states.clone());
    let remove_object_states = random_remove_object_states();

    //Merge new object states and update object states
    new_object_states.append(&mut update_object_states);
    indexer_store.persist_or_update_object_states(new_object_states.clone())?;
    indexer_store.delete_object_states(remove_object_states)?;

    // test for querying batch objects with filter ObjectStateFilter::ObjectId
    let num_objs = new_object_ids.len();
    let filter = ObjectStateFilter::ObjectId(new_object_ids);
    let query_object_states =
        indexer_reader.query_object_states_with_filter(filter, None, num_objs, true)?;
    assert_eq!(query_object_states.len(), num_objs);

    Ok(())
}

#[test]
fn test_object_type_query() -> Result<()> {
    let registry_service = RegistryService::default();
    let tmpdir = moveos_config::temp_dir();
    let indexer_db = tmpdir.path().join(DEFAULT_DB_INDEXER_SUBDIR);
    let indexer_store =
        IndexerStore::new(indexer_db.clone(), &registry_service.default_registry())?;
    let indexer_reader = IndexerReader::new(indexer_db, &registry_service.default_registry())?;
    let object_id = ObjectID::random();
    let owner = AccountAddress::random();
    let coin_store_obj = ObjectEntity::new(
        object_id.clone(),
        owner,
        0,
        Some(H256::random()),
        0,
        0,
        0,
        CoinStore::<GasCoin>::new(100u64.into(), false),
    );
    let raw_obj = coin_store_obj.into_state();
    let state = IndexerObjectState::new(raw_obj.metadata, 0, 0);
    let object_states = vec![state];
    indexer_store.persist_or_update_object_states(object_states.clone())?;
    // filter by exact object type
    let filter = ObjectStateFilter::ObjectType(CoinStore::<GasCoin>::struct_tag());
    let query_object_states =
        indexer_reader.query_object_states_with_filter(filter, None, 1, true)?;
    assert_eq!(query_object_states.len(), 1);
    // filter by object type and owner
    let filter = ObjectStateFilter::ObjectTypeWithOwner {
        object_type: CoinStore::<GasCoin>::struct_tag(),
        filter_out: false,
        owner,
    };
    let query_object_states =
        indexer_reader.query_object_states_with_filter(filter, None, 1, true)?;
    assert_eq!(query_object_states.len(), 1);
    // filter by object owner
    let filter = ObjectStateFilter::Owner(owner);
    let query_object_states =
        indexer_reader.query_object_states_with_filter(filter, None, 1, true)?;
    assert_eq!(query_object_states.len(), 1);
    // filter by object type without type params
    let filter = ObjectStateFilter::ObjectType(CoinStore::struct_tag_without_coin_type());
    let query_object_states =
        indexer_reader.query_object_states_with_filter(filter, None, 1, true)?;
    assert_eq!(query_object_states.len(), 1);
    Ok(())
}

#[test]
fn test_escape_transaction() -> Result<()> {
    let registry_service = RegistryService::default();
    let tmpdir = moveos_config::temp_dir();
    let indexer_db = tmpdir.path().join(DEFAULT_DB_INDEXER_SUBDIR);
    let indexer_store =
        IndexerStore::new(indexer_db.clone(), &registry_service.default_registry())?;
    let indexer_reader = IndexerReader::new(indexer_db, &registry_service.default_registry())?;

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
        root: ObjectMeta::genesis_root(),
        ctx: tx_context,
        action: move_action,
    };

    let mut indexer_transaction = IndexerTransaction::new(
        random_transaction,
        random_execution_info,
        random_moveos_tx.action.into(),
        random_moveos_tx.ctx.clone(),
    )?;
    // construct escape field
    let quotes = "Executed: ''There is no escape'";
    indexer_transaction.status = quotes.to_string();
    let transactions = vec![indexer_transaction];
    indexer_store.persist_transactions(transactions)?;

    let filter = TransactionFilter::Sender(random_moveos_tx.ctx.sender.into());
    let query_transactions =
        indexer_reader.query_transactions_with_filter(filter, None, 1, true)?;
    assert_eq!(query_transactions.len(), 1);
    Ok(())
}
