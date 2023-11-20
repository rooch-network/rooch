// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::indexer_reader::IndexerReader;
use crate::store::traits::IndexerStoreTrait;
use crate::types::{IndexedEvent, IndexedTransaction};
use crate::IndexerStore;
use anyhow::Result;
use ethers::types::{Bytes, U256};
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::ModuleId;
use move_core_types::vm_status::KeptVMStatus;
use moveos_types::h256::H256;
use moveos_types::move_types::{random_identity, random_struct_tag, random_type_tag, FunctionId};
use moveos_types::moveos_std::event::{Event, EventID};
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::transaction::{
    FunctionCall, MoveAction, ScriptCall, TransactionExecutionInfo, VerifiedMoveAction,
    VerifiedMoveOSTransaction,
};
use rand::{thread_rng, Rng};
use rooch_config::indexer_config::ROOCH_INDEXER_DB_FILENAME;
use rooch_types::address::{RoochAddress, RoochSupportedAddress};
use rooch_types::indexer::event_filter::EventFilter;
use rooch_types::indexer::transaction_filter::TransactionFilter;
use rooch_types::transaction::authenticator::Authenticator;
use rooch_types::transaction::ethereum::EthereumTransaction;
use rooch_types::transaction::rooch::{RoochTransaction, RoochTransactionData};
use rooch_types::transaction::{TransactionSequenceInfo, TypedTransaction};

fn random_bytes() -> Vec<u8> {
    H256::random().0.to_vec()
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
