// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::event_view::IndexerEventIDView;
use super::{
    HumanReadableDisplay, IndexerStateIDView, ObjectIDView, StateChangeSetWithTxOrderView,
};
use crate::jsonrpc_types::account_view::BalanceInfoView;
use crate::jsonrpc_types::btc::ord::InscriptionStateView;
use crate::jsonrpc_types::btc::utxo::UTXOStateView;
use crate::jsonrpc_types::event_view::{EventView, IndexerEventView};
use crate::jsonrpc_types::field_view::IndexerFieldView;
use crate::jsonrpc_types::transaction_view::TransactionWithInfoView;
use crate::jsonrpc_types::{
    move_types::{MoveActionTypeView, MoveActionView},
    BytesView, IndexerObjectStateView, StateKVView, StrView, StructTagView,
};
use move_core_types::language_storage::StructTag;
use move_core_types::u256::U256;
use moveos_types::moveos_std::event::EventHandle;
use moveos_types::moveos_std::object::ObjectID;
use rooch_types::framework::coin::CoinInfo;
use rooch_types::transaction::rooch::RoochTransaction;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::string::String;

pub type EventPageView = PageView<EventView, StrView<u64>>;
pub type TransactionWithInfoPageView = PageView<TransactionWithInfoView, StrView<u64>>;
pub type StatePageView = PageView<StateKVView, String>;
pub type BalanceInfoPageView = PageView<BalanceInfoView, IndexerStateIDView>;
pub type IndexerEventPageView = PageView<IndexerEventView, IndexerEventIDView>;

pub type IndexerObjectStatePageView = PageView<IndexerObjectStateView, IndexerStateIDView>;

pub type UTXOPageView = PageView<UTXOStateView, IndexerStateIDView>;
pub type InscriptionPageView = PageView<InscriptionStateView, IndexerStateIDView>;
pub type StateChangeSetPageView = PageView<StateChangeSetWithTxOrderView, StrView<u64>>;

pub type FieldPageView = PageView<IndexerFieldView, StrView<u64>>;

/// `next_cursor` points to the last item in the page;
/// Reading with `next_cursor` will start from the next item after `next_cursor` if
/// `next_cursor` is `Some`, otherwise it will start from the first item.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct PageView<T, C> {
    pub data: Vec<T>,
    pub next_cursor: Option<C>,
    pub has_next_page: bool,
}

impl<T, C> HumanReadableDisplay for PageView<T, C>
where
    T: HumanReadableDisplay,
    C: std::fmt::Display,
{
    fn to_human_readable_string(&self, verbose: bool, indent: usize) -> String {
        let _ = verbose;
        format!(
            r#"{indent}Data:
{indent}{}

{indent}Next cursor:
{indent}    {}

{indent}Has next page: {:?}"#,
            self.data.to_human_readable_string(verbose, 4),
            self.next_cursor
                .as_ref()
                .map_or("None".to_string(), |c| c.to_string()),
            self.has_next_page,
            indent = " ".repeat(indent)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionView {
    pub sequence_number: StrView<u64>,
    pub sender: String,
    pub sender_bitcoin_address: Option<String>,
    pub action_type: MoveActionTypeView,
    pub action: MoveActionView,
    pub raw: BytesView,
    pub chain_id: StrView<u64>,
    pub max_gas_amount: StrView<u64>,
}

impl TransactionView {
    pub fn new_from_rooch_transaction(
        transaction: RoochTransaction,
        sender_bitcoin_address: Option<String>,
    ) -> Self {
        Self {
            sequence_number: transaction.sequence_number().into(),
            sender: transaction.sender().to_string(),
            sender_bitcoin_address,
            action: transaction.action().clone().into(),
            action_type: transaction.action().clone().into(),
            raw: transaction.encode().into(),
            chain_id: transaction.chain_id().into(),
            max_gas_amount: transaction.max_gas_amount().into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoinInfoView {
    pub coin_type: StructTagView,
    pub name: String,
    pub symbol: String,
    pub icon_url: Option<String>,
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
            icon_url: coin_info.icon_url(),
            decimals: coin_info.decimals(),
            supply: StrView(coin_info.supply()),
        }
    }
}

// To compatiable with the old format RPC request
pub type StructTagOrObjectIDView = String;
pub type EnumStructTagOrObjectIDView = StrView<EnumStructTagOrObjectID>;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnumStructTagOrObjectID {
    StructTag(StructTag),
    ObjectID(ObjectID),
}

impl From<StructTag> for EnumStructTagOrObjectID {
    fn from(item: StructTag) -> Self {
        Self::StructTag(item)
    }
}

impl From<ObjectID> for EnumStructTagOrObjectID {
    fn from(item: ObjectID) -> Self {
        Self::ObjectID(item)
    }
}

impl FromStr for EnumStructTagOrObjectIDView {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try to parse as StructTag first
        if let Ok(struct_tag) = StructTag::from_str(s) {
            return Ok(StrView(EnumStructTagOrObjectID::StructTag(struct_tag)));
        }
        // If not a valid StructTag, try to parse as ObjectID
        if let Ok(object_id) = ObjectID::from_str(s) {
            return Ok(StrView(EnumStructTagOrObjectID::ObjectID(object_id)));
        }
        Err(anyhow::anyhow!(
            "Failed to parse as either StructTag or ObjectID"
        ))
    }
}

impl From<EnumStructTagOrObjectIDView> for ObjectIDView {
    fn from(view: EnumStructTagOrObjectIDView) -> Self {
        match view.0 {
            EnumStructTagOrObjectID::StructTag(struct_tag) => {
                StrView(EventHandle::derive_event_handle_id(&struct_tag))
            }
            EnumStructTagOrObjectID::ObjectID(object_id) => StrView(object_id),
        }
    }
}

impl std::fmt::Display for EnumStructTagOrObjectIDView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            EnumStructTagOrObjectID::StructTag(struct_tag) => {
                // EventHandle::derive_event_handle_id(&struct_tag).into()
                write!(f, "{}", struct_tag.to_canonical_string())
            }
            EnumStructTagOrObjectID::ObjectID(object_id) => {
                write!(f, "{}", object_id)
            }
        }
    }
}
