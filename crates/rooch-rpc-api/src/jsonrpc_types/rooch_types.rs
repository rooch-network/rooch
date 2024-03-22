// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::account_view::BalanceInfoView;
use crate::jsonrpc_types::btc::ord::InscriptionStateView;
use crate::jsonrpc_types::btc::utxo::UTXOStateView;
use crate::jsonrpc_types::event_view::{EventView, IndexerEventView};
use crate::jsonrpc_types::transaction_view::TransactionWithInfoView;
use crate::jsonrpc_types::{
    move_types::{MoveActionTypeView, MoveActionView},
    BytesView, IndexerFieldStateView, IndexerObjectStateView, IndexerTableChangeSetView,
    StateKVView, StrView, StructTagView,
};
use move_core_types::u256::U256;
use rooch_types::framework::coin::CoinInfo;
use rooch_types::indexer::event_filter::IndexerEventID;
use rooch_types::indexer::state::IndexerStateID;
use rooch_types::transaction::rooch::RoochTransaction;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::string::String;

pub type EventPageView = PageView<EventView, u64>;
pub type TransactionWithInfoPageView = PageView<TransactionWithInfoView, u64>;
pub type StatePageView = PageView<StateKVView, String>;
pub type BalanceInfoPageView = PageView<BalanceInfoView, String>;
pub type IndexerEventPageView = PageView<IndexerEventView, IndexerEventID>;
pub type IndexerTableChangeSetPageView = PageView<IndexerTableChangeSetView, IndexerStateID>;

pub type IndexerObjectStatePageView = PageView<IndexerObjectStateView, IndexerStateID>;
pub type IndexerFieldStatePageView = PageView<IndexerFieldStateView, IndexerStateID>;

pub type UTXOPageView = PageView<UTXOStateView, IndexerStateID>;
pub type InscriptionPageView = PageView<InscriptionStateView, IndexerStateID>;

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
pub struct TransactionView {
    pub sequence_number: u64,
    pub sender: String,
    pub action_type: MoveActionTypeView,
    pub action: MoveActionView,
    pub raw: BytesView,
}

impl From<RoochTransaction> for TransactionView {
    fn from(transaction: RoochTransaction) -> Self {
        Self {
            sequence_number: transaction.sequence_number(),
            sender: transaction.sender().to_string(),
            action: transaction.action().clone().into(),
            action_type: transaction.action().clone().into(),
            raw: transaction.encode().into(),
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
