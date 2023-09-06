// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    move_types::{MoveActionTypeView, MoveActionView},
    AnnotatedMoveStructView, AnnotatedMoveValueView, AnnotatedStateView, EventView, H256View,
    StateView, StrView, StructTagView, TransactionExecutionInfoView,
};
use anyhow::bail;
use anyhow::Result;
use move_core_types::u256::U256;
use moveos_types::event::AnnotatedMoveOSEvent;
use moveos_types::state::MoveState;
use rooch_types::transaction::{AbstractTransaction, TransactionType, TypedTransaction};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

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

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinView {
    value: StrView<U256>,
}

impl CoinView {
    pub fn new(value: StrView<U256>) -> Self {
        CoinView { value }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedCoinView {
    type_: StructTagView,
    value: CoinView,
}

impl AnnotatedCoinView {
    pub fn new(type_: StructTagView, value: CoinView) -> Self {
        AnnotatedCoinView { type_, value }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundCoinStoreView {
    coin: AnnotatedCoinView,
    frozen: bool,
}

impl CompoundCoinStoreView {
    pub fn new(coin: AnnotatedCoinView, frozen: bool) -> Self {
        CompoundCoinStoreView { coin, frozen }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedCoinStoreView {
    type_: StructTagView,
    value: CompoundCoinStoreView,
}

impl AnnotatedCoinStoreView {
    pub fn new(type_: StructTagView, value: CompoundCoinStoreView) -> Self {
        AnnotatedCoinStoreView { type_, value }
    }

    /// Create a new AnnotatedCoinStoreView from a AnnotatedMoveValueView
    pub fn new_from_annotated_move_value_view(
        annotated_move_value_view: AnnotatedMoveValueView,
    ) -> Result<Self> {
        match annotated_move_value_view {
            AnnotatedMoveValueView::Struct(annotated_struct_view) => {
                let annotated_coin_store_type = annotated_struct_view.type_;
                let mut fields = annotated_struct_view.value.into_iter();
                let annotated_coin = match fields.next().expect("CoinStore should have coin field")
                {
                    (field_name, AnnotatedMoveValueView::Struct(filed_value)) => {
                        debug_assert!(
                            field_name.as_str() == "coin",
                            "CoinStore coin field name should be coin"
                        );

                        let coin_type_ = filed_value.type_;

                        let mut inner_fields = filed_value.value.into_iter();
                        let coin_value = match inner_fields
                            .next()
                            .expect("CoinValue should have value field")
                        {
                            (field_name, AnnotatedMoveValueView::Bytes(inner_filed_value)) => {
                                debug_assert!(
                                    field_name.as_str() == "value",
                                    "CoinValue value field name should be value"
                                );
                                U256::from_bytes(inner_filed_value.0.as_slice())?
                            }
                            (field_name, AnnotatedMoveValueView::U64(inner_filed_value)) => {
                                debug_assert!(
                                    field_name.as_str() == "value",
                                    "CoinValue value field name should be value"
                                );
                                U256::from(inner_filed_value.0)
                            }
                            (field_name, AnnotatedMoveValueView::U128(inner_filed_value)) => {
                                debug_assert!(
                                    field_name.as_str() == "value",
                                    "CoinValue value field name should be value"
                                );
                                U256::from(inner_filed_value.0)
                            }
                            (field_name, AnnotatedMoveValueView::U256(inner_filed_value)) => {
                                debug_assert!(
                                    field_name.as_str() == "value",
                                    "CoinValue value field name should be value"
                                );
                                inner_filed_value.0
                            }
                            _ => bail!("CoinValue value field should be value"),
                        };
                        let coin = CoinView {
                            value: StrView(coin_value),
                        };
                        AnnotatedCoinView {
                            type_: coin_type_,
                            value: coin,
                        }
                    }
                    _ => bail!("CoinStore coin field should be struct"),
                };
                let frozen = match fields.next().expect("CoinStore should have frozen field") {
                    (field_name, AnnotatedMoveValueView::Bool(filed_value)) => {
                        debug_assert!(
                            field_name.as_str() == "frozen",
                            "CoinStore field name should be frozen"
                        );
                        filed_value
                    }
                    _ => bail!("CoinStore frozen field should be bool"),
                };
                let compose_coin_store = CompoundCoinStoreView {
                    coin: annotated_coin,
                    frozen,
                };

                let annotated_coin_store_view = AnnotatedCoinStoreView {
                    type_: annotated_coin_store_type,
                    value: compose_coin_store,
                };

                Ok(annotated_coin_store_view)
            }
            _ => bail!("CoinValue value field should be value"),
        }
    }

    pub fn get_coin_type(&self) -> StructTagView {
        self.value.coin.type_.clone()
    }

    pub fn get_coin_value(&self) -> StrView<U256> {
        self.value.coin.value.value
    }
}
