// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    move_types::{MoveActionTypeView, MoveActionView},
    AnnotatedMoveStructView, AnnotatedStateView, EventView, H256View, StateView, StrView,
    TransactionExecutionInfoView,
};
use moveos_types::event::AnnotatedMoveOSEvent;
use rooch_types::transaction::{AbstractTransaction, TransactionType, TypedTransaction};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::AccountAddressView;

pub type EventPageView = PageView<Option<AnnotatedEventView>, u64>;
pub type TransactionInfoPageView = PageView<Option<TransactionExecutionInfoView>, u128>;
pub type ListStatesPageView = PageView<Option<StateView>, StrView<Vec<u8>>>;
pub type ListAnnotatedStatesPageView = PageView<Option<AnnotatedStateView>, StrView<Vec<u8>>>;

/// `next_cursor` points to the last item in the page;
/// Reading with `next_cursor` will start from the next item after `next_cursor` if
/// `next_cursor` is `Some`, otherwise it will start from the first item.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct PageView<T, C> {
    pub data: Vec<T>,
    pub next_cursor: Option<C>,
    pub has_next_page: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TransactionTypeView {
    Rooch,
    Ethereum,
}

impl From<TransactionType> for TransactionTypeView {
    fn from(tt: TransactionType) -> Self {
        match tt {
            TransactionType::Rooch => Self::Rooch,
            TransactionType::Ethereum => Self::Ethereum,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionView {
    pub transaction_type: TransactionTypeView,
    pub sequence_number: u64,
    // TBD: how to represent the sender.
    pub sender: String,
    pub action_type: MoveActionTypeView,
    pub action: MoveActionView,
    pub raw: StrView<Vec<u8>>,
}

impl From<TypedTransaction> for TransactionView {
    fn from(transaction: TypedTransaction) -> Self {
        let transaction_type = transaction.transaction_type();
        match transaction {
            TypedTransaction::Rooch(rooch) => Self {
                transaction_type: transaction_type.into(),
                sequence_number: rooch.sequence_number(),
                sender: rooch.sender().to_string(),
                action: rooch.action().clone().into(),
                action_type: rooch.action().clone().into(),
                raw: rooch.encode().into(),
            },
            TypedTransaction::Ethereum(eth) => Self {
                transaction_type: transaction_type.into(),
                sequence_number: eth.0.nonce.as_u64(),
                sender: eth.0.from.to_string(),
                action: eth.decode_calldata_to_action().unwrap().into(),
                action_type: eth.decode_calldata_to_action().unwrap().into(),
                raw: eth.encode().into(),
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AnnotatedEventView {
    pub event: EventView,
    pub sender: AccountAddressView,
    pub tx_hash: Option<H256View>,
    pub timestamp_ms: Option<u64>,
    // pub block_height: Option<u64>,
    pub parsed_event_data: AnnotatedMoveStructView,
}

impl From<AnnotatedMoveOSEvent> for AnnotatedEventView {
    fn from(event: AnnotatedMoveOSEvent) -> Self {
        AnnotatedEventView {
            event: event.event.into(),
            sender: event.sender.into(),
            tx_hash: event.tx_hash.map(|h256| h256.into()),
            timestamp_ms: event.timestamp_ms,
            // block_height: event.block_height,
            parsed_event_data: event.parsed_event_data.into(),
        }
    }
}

// impl From<AnnotatedEventView> for AnnotatedMoveOSEvent {
//     fn from(event: AnnotatedEventView) -> Self {
//         AnnotatedMoveOSEvent {
//             event: event.into(),
//             sender: AccountAddress::try_from(event.sender).unwrap(),
//             tx_hash: event.tx_hash,
//             timestamp_ms: event.timestamp_ms,
//             // block_height: event.block_height,
//             parsed_event_data: event.parsed_event_data.into(),
//         }
//     }
// }
