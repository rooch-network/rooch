// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::subscription_handler::SubscriptionHandler;
use move_core_types::account_address::AccountAddress;
use move_core_types::value::MoveTypeLayout;
use move_core_types::{ident_str, language_storage::StructTag};
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::state::MoveStructState;
use moveos_types::test_utils::random_event;
use moveos_types::transaction::TransactionExecutionInfo;
use prometheus::Registry;
use rooch_types::indexer::event::EventFilter;
use rooch_types::indexer::transaction::TransactionFilter;
use rooch_types::test_utils::random_ledger_transaction;
use rooch_types::transaction::TransactionWithInfo;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;
use tokio::time::timeout;
use tokio_stream::StreamExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestEvent {
    creator: AccountAddress,
    name: MoveString,
    data: Vec<u64>,
}

#[allow(dead_code)]
impl TestEvent {
    fn type_layout() -> StructTag {
        StructTag {
            address: AccountAddress::from_hex_literal("0x42").unwrap(),
            module: ident_str!("test").to_owned(),
            name: ident_str!("test_event").to_owned(),
            type_params: vec![],
        }
    }

    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            MoveTypeLayout::Address,
            MoveTypeLayout::Struct(MoveString::struct_layout()),
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U64)),
        ])
    }
}

#[tokio::test]
async fn test_event_subscription() {
    let registry = Registry::new();
    let handler = SubscriptionHandler::new(&registry);

    // Create a test event
    // let test_event1 = TestEvent {
    //     creator: AccountAddress::random(),
    //     name: "test_event1".into(),
    //     data: vec![1, 2, 3],
    // };
    // let test_event2 = TestEvent {
    //     creator: AccountAddress::random(),
    //     name: "test_event2".into(),
    //     data: vec![4, 5, 6],
    // };
    // let test_event3 = TestEvent {
    //     creator: AccountAddress::random(),
    //     name: "test_event3".into(),
    //     data: vec![7, 8, 9],
    // };

    // Create an event filter that matches all
    let event_filter = EventFilter::All;
    // let event_filter = EventFilter::EventType(TestEvent::type_layout());

    // Subscribe to events
    let mut event_stream = handler.subscribe_events(event_filter);

    // Create a mock transaction and context
    let ledger_tx = random_ledger_transaction();
    let tx_execution_info = TransactionExecutionInfo::random();
    let tx = TransactionWithInfo::new(ledger_tx, tx_execution_info);
    let ctx = TxContext::random_for_testing_only();

    // Create random Event
    let event1 = random_event();
    let event2 = random_event();
    let event3 = random_event();
    let events = vec![event1, event2, event3];
    // let events = vec![test_event1, test_event2, test_event3];

    // Process the transaction with events
    handler.process_tx_with_events(tx, events, ctx).unwrap();

    // Try to receive the event with a timeout
    let received_event = timeout(Duration::from_secs(1), event_stream.next()).await;
    assert!(
        received_event.is_ok(),
        "Should receive event within timeout"
    );
}

#[tokio::test]
async fn test_transaction_subscription() {
    let registry = Registry::new();
    let handler = SubscriptionHandler::new(&registry);

    // Create a transaction filter that matches all transactions in a time range
    let tx_filter = TransactionFilter::TimeRange {
        start_time: 0,
        end_time: u64::MAX,
    };

    // Subscribe to transactions
    let mut tx_stream = handler.subscribe_transactions(tx_filter);

    // Create and process a test transaction
    let ledger_tx = random_ledger_transaction();
    let tx_execution_info = TransactionExecutionInfo::random();
    let tx = TransactionWithInfo::new(ledger_tx, tx_execution_info);
    let ctx = TxContext::random_for_testing_only();

    handler
        .process_tx_with_events(tx.clone(), vec![], ctx)
        .unwrap();

    // Try to receive the transaction with a timeout
    let received_tx = timeout(Duration::from_secs(1), tx_stream.next()).await;
    assert!(
        received_tx.is_ok(),
        "Should receive transaction within timeout"
    );

    if let Ok(Some(received)) = received_tx {
        assert_eq!(
            received.transaction.sequence_info.tx_order,
            tx.transaction.sequence_info.tx_order
        );
    }
}

#[tokio::test]
async fn test_multiple_subscribers() {
    let registry = Registry::new();
    let handler = SubscriptionHandler::new(&registry);

    // Create multiple subscribers
    // let event_filter = EventFilter::EventType(TestEvent::type_layout());
    let event_filter = EventFilter::All;
    let mut stream1 = handler.subscribe_events(event_filter.clone());
    let mut stream2 = handler.subscribe_events(event_filter);

    // Create and process a test event
    // let _test_event = TestEvent {
    //     creator: AccountAddress::random(),
    //     name: "test_event".into(),
    //     data: vec![1, 2, 3],
    // };

    let ledger_tx = random_ledger_transaction();
    let tx_execution_info = TransactionExecutionInfo::random();
    let tx = TransactionWithInfo::new(ledger_tx, tx_execution_info);
    let ctx = TxContext::random_for_testing_only();

    // Create random Event
    let event1 = random_event();
    let event2 = random_event();
    let event3 = random_event();
    let events = vec![event1, event2, event3];

    handler.process_tx_with_events(tx, events, ctx).unwrap();

    // Both streams should receive the event
    let received1 = timeout(Duration::from_secs(1), stream1.next()).await;
    let received2 = timeout(Duration::from_secs(1), stream2.next()).await;

    assert!(
        received1.is_ok() && received2.is_ok(),
        "Both streams should receive events"
    );
}

// #[tokio::test]
// async fn test_filter_matching() {
//     let registry = Registry::new();
//     let handler = SubscriptionHandler::new(&registry);
//
//     // Create two different event filters
//     let matching_filter = EventFilter::EventType(TestEvent::type_layout());
//     let non_matching_filter = EventFilter::EventType(StructTag {
//         address: AccountAddress::from_hex_literal("0x43").unwrap(),
//         module: ident_str!("different").to_owned(),
//         name: ident_str!("different_event").to_owned(),
//         type_params: vec![],
//     });
//
//     let mut matching_stream = handler.subscribe_events(matching_filter);
//     let mut non_matching_stream = handler.subscribe_events(non_matching_filter);
//
//     // Create and process a test event
//     let test_event = TestEvent {
//         creator: AccountAddress::random(),
//         name: "test_event".into(),
//         data: vec![1, 2, 3],
//     };
//
//     let tx = RoochTransaction::new_for_test(());
//     let ctx = TxContext::new_for_test(());
//     let event = Event::new_for_test(test_event);
//
//     handler
//         .process_tx_with_events(TransactionWithInfo::new_for_test(tx), vec![event], ctx)
//         .unwrap();
//
//     // The matching stream should receive the event
//     let matching_received = timeout(Duration::from_secs(1), matching_stream.next()).await;
//     assert!(
//         matching_received.is_ok(),
//         "Matching stream should receive event"
//     );
//
//     // The non-matching stream should timeout
//     let non_matching_received =
//         timeout(Duration::from_millis(100), non_matching_stream.next()).await;
//     assert!(
//         non_matching_received.is_err(),
//         "Non-matching stream should not receive event"
//     );
// }
