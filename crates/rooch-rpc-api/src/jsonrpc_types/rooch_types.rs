// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::account_view::BalanceInfoView;
use crate::jsonrpc_types::event_view::{EventView, IndexerEventView};
use crate::jsonrpc_types::transaction_view::TransactionWithInfoView;
use crate::jsonrpc_types::{
    move_types::{MoveActionTypeView, MoveActionView},
    BytesView, StateView, StrView, StructTagView,
};
use move_core_types::u256::U256;
use rooch_types::framework::coin::CoinInfo;
use rooch_types::indexer::event_filter::IndexerEventID;
use rooch_types::transaction::{AbstractTransaction, TransactionType, TypedTransaction};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::string::String;

pub type EventPageView = PageView<EventView, u64>;
pub type TransactionWithInfoPageView = PageView<TransactionWithInfoView, u64>;
pub type StatesPageView = PageView<StateView, BytesView>;
pub type BalanceInfoPageView = PageView<BalanceInfoView, BytesView>;
pub type IndexerEventPageView = PageView<IndexerEventView, IndexerEventID>;

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
    pub raw: BytesView,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoinInfoView {
    pub coin_type: StructTagView,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub supply: StrView<U256>,
}

impl<CoinType> From<CoinInfo<CoinType>> for CoinInfoView {
    fn from(coin_info: CoinInfo<CoinType>) -> Self {
        Self {
            //We convert the coin_type to Coin Type tag here
            //Because the coin_type string is the `to_canonical_string` of the StructTag
            //It's not the same as the StructTagView string
            coin_type: coin_info.coin_type_tag().into(),
            name: coin_info.name(),
            symbol: coin_info.symbol(),
            decimals: coin_info.decimals(),
            supply: StrView(coin_info.supply()),
        }
    }
}
