// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::aggregate_service::AggregateService;
use crate::service::rpc_service::RpcService;
use anyhow::Result;
use jsonrpsee::{
    core::{async_trait, Error as JsonRpcError, RpcResult},
    RpcModule,
};
use move_core_types::account_address::AccountAddress;
use moveos_types::{
    access_path::AccessPath,
    h256::H256,
    moveos_std::{
        display::{get_object_display_id, get_resource_display_id, RawDisplay},
        object::ObjectEntity,
    },
    state::{AnnotatedKeyState, AnnotatedState, KeyState},
};
use rooch_rpc_api::jsonrpc_types::event_view::{EventFilterView, EventView, IndexerEventView};
use rooch_rpc_api::jsonrpc_types::transaction_view::TransactionFilterView;
use rooch_rpc_api::jsonrpc_types::{
    account_view::BalanceInfoView, GlobalStateFilterView, IndexerEventPageView,
    IndexerGlobalStatePageView, IndexerGlobalStateView, IndexerTableChangeSetPageView,
    IndexerTableChangeSetView, IndexerTableStatePageView, IndexerTableStateView, KeyStateView,
    StateKVView, StateOptions, StateSyncFilterView, TableStateFilterView, TxOptions,
};
use rooch_rpc_api::jsonrpc_types::{transaction_view::TransactionWithInfoView, EventOptions};
use rooch_rpc_api::jsonrpc_types::{
    AccessPathView, AccountAddressView, BalanceInfoPageView, DisplayFieldsView, EventPageView,
    ExecuteTransactionResponseView, FunctionCallView, H256View, StatePageView, StateView, StrView,
    StructTagView, TransactionWithInfoPageView,
};
use rooch_rpc_api::{api::rooch_api::RoochAPIServer, api::DEFAULT_RESULT_LIMIT};
use rooch_rpc_api::{
    api::{RoochRpcModule, DEFAULT_RESULT_LIMIT_USIZE},
    jsonrpc_types::AnnotatedFunctionResultView,
};
use rooch_rpc_api::{
    api::{MAX_RESULT_LIMIT, MAX_RESULT_LIMIT_USIZE},
    jsonrpc_types::BytesView,
};
use rooch_types::indexer::event_filter::IndexerEventID;
use rooch_types::indexer::state::IndexerStateID;
use rooch_types::transaction::rooch::RoochTransaction;
use rooch_types::{address::MultiChainAddress, multichain_id::RoochMultiChainID};
use std::cmp::min;
use std::str::FromStr;
use tracing::info;

pub struct RoochServer {
    rpc_service: RpcService,
    aggregate_service: AggregateService,
}

impl RoochServer {
    pub fn new(rpc_service: RpcService, aggregate_service: AggregateService) -> Self {
        Self {
            rpc_service,
            aggregate_service,
        }
    }

    async fn get_display_fields_and_render(
        &self,
        states: Vec<&AnnotatedState>,
        is_object: bool,
    ) -> Result<Vec<Option<DisplayFieldsView>>> {
        let mut display_ids = vec![];
        let mut displayable_states = vec![];
        for s in &states {
            displayable_states.push(if is_object {
                if let Some(tag) = s.state.get_object_struct_tag() {
                    display_ids.push(get_object_display_id(tag.into()));
                    true
                } else {
                    false
                }
            } else if let Some(tag) = s.state.get_resource_struct_tag() {
                display_ids.push(get_resource_display_id(tag.into()));
                true
            } else {
                false
            });
        }
        // get display fields
        let path = AccessPath::objects(display_ids);
        let mut display_fields = self
            .rpc_service
            .get_states(path)
            .await?
            .into_iter()
            .map(|option_s| {
                option_s
                    .map(|s| s.as_object_uncheck::<RawDisplay>())
                    .transpose()
            })
            .collect::<Result<Vec<Option<ObjectEntity<RawDisplay>>>>>()?;
        display_fields.reverse();

        let mut display_field_views = vec![];
        for (annotated_s, displayable) in states.into_iter().zip(displayable_states) {
            display_field_views.push(if displayable {
                debug_assert!(
                    !display_fields.is_empty(),
                    "Display fields should not be empty"
                );
                display_fields
                    .pop()
                    .unwrap()
                    .map(|obj| DisplayFieldsView::new(obj.value.render(&annotated_s.decoded_value)))
            } else {
                None
            });
        }
        Ok(display_field_views)
    }
}

#[async_trait]
impl RoochAPIServer for RoochServer {
    async fn get_chain_id(&self) -> RpcResult<StrView<u64>> {
        let chain_id = self.rpc_service.get_chain_id();
        Ok(StrView(chain_id))
    }

    async fn send_raw_transaction(&self, payload: BytesView) -> RpcResult<H256View> {
        info!("send_raw_transaction payload: {:?}", payload);
        let tx = bcs::from_bytes::<RoochTransaction>(&payload.0).map_err(anyhow::Error::from)?;
        info!("send_raw_transaction tx: {:?}", tx);

        let hash = tx.tx_hash();
        self.rpc_service.quene_tx(tx).await?;
        Ok(hash.into())
    }

    async fn execute_raw_transaction(
        &self,
        payload: BytesView,
        tx_options: Option<TxOptions>,
    ) -> RpcResult<ExecuteTransactionResponseView> {
        let tx_options = tx_options.unwrap_or_default();
        let tx = bcs::from_bytes::<RoochTransaction>(&payload.0).map_err(anyhow::Error::from)?;
        let tx_response = self.rpc_service.execute_tx(tx).await?;

        let result = if tx_options.with_output {
            ExecuteTransactionResponseView::from(tx_response)
        } else {
            ExecuteTransactionResponseView::new_without_output(tx_response)
        };
        Ok(result)
    }

    async fn execute_view_function(
        &self,
        function_call: FunctionCallView,
    ) -> RpcResult<AnnotatedFunctionResultView> {
        Ok(self
            .rpc_service
            .execute_view_function(function_call.into())
            .await?
            .into())
    }

    async fn get_states(
        &self,
        access_path: AccessPathView,
        state_option: Option<StateOptions>,
    ) -> RpcResult<Vec<Option<StateView>>> {
        let state_option = state_option.unwrap_or_default();
        let show_display =
            state_option.show_display && (access_path.0.is_object() || access_path.0.is_resource());

        let state_views = if state_option.decode || show_display {
            let is_object = access_path.0.is_object();
            let states = self
                .rpc_service
                .get_annotated_states(access_path.into())
                .await?;

            if show_display {
                let valid_states = states.iter().filter_map(|s| s.as_ref()).collect::<Vec<_>>();
                let mut valid_display_field_views = self
                    .get_display_fields_and_render(valid_states, is_object)
                    .await?;
                valid_display_field_views.reverse();
                states
                    .into_iter()
                    .map(|option_annotated_s| {
                        option_annotated_s.map(|annotated_s| {
                            debug_assert!(
                                !valid_display_field_views.is_empty(),
                                "display fields should not be empty"
                            );
                            let display_view = valid_display_field_views.pop().unwrap();
                            StateView::from(annotated_s).with_display_fields(display_view)
                        })
                    })
                    .collect()
            } else {
                states.into_iter().map(|s| s.map(StateView::from)).collect()
            }
        } else {
            self.rpc_service
                .get_states(access_path.into())
                .await?
                .into_iter()
                .map(|s| s.map(StateView::from))
                .collect()
        };
        Ok(state_views)
    }

    async fn list_states(
        &self,
        access_path: AccessPathView,
        cursor: Option<String>,
        limit: Option<StrView<usize>>,
        state_option: Option<StateOptions>,
    ) -> RpcResult<StatePageView> {
        let state_option = state_option.unwrap_or_default();
        let show_display =
            state_option.show_display && (access_path.0.is_object() || access_path.0.is_resource());

        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let cursor_of = match cursor.clone() {
            Some(key_state_str) => Some(KeyState::from_str(key_state_str.as_str())?),
            None => None,
        };
        let mut data: Vec<StateKVView> = if state_option.decode || show_display {
            let is_object = access_path.0.is_object();
            let (key_states, states): (Vec<AnnotatedKeyState>, Vec<AnnotatedState>) = self
                .rpc_service
                .list_annotated_states(access_path.into(), cursor_of, limit_of + 1)
                .await?
                .into_iter()
                .unzip();
            let state_refs: Vec<&AnnotatedState> = states.iter().collect();
            if show_display {
                let display_field_views = self
                    .get_display_fields_and_render(state_refs, is_object)
                    .await?;
                key_states
                    .into_iter()
                    .zip(states)
                    .zip(display_field_views)
                    .map(|((key_state, state), display_field_view)| {
                        StateKVView::new(
                            KeyStateView::from(key_state),
                            StateView::from(state).with_display_fields(display_field_view),
                        )
                    })
                    .collect::<Vec<_>>()
            } else {
                key_states
                    .into_iter()
                    .zip(states)
                    .map(|(key_state, state)| {
                        StateKVView::new(KeyStateView::from(key_state), StateView::from(state))
                    })
                    .collect::<Vec<_>>()
            }
        } else {
            self.rpc_service
                .list_states(access_path.into(), cursor_of, limit_of + 1)
                .await?
                .into_iter()
                .map(|(key_state, state)| {
                    StateKVView::new(KeyStateView::from(key_state), StateView::from(state))
                })
                .collect::<Vec<_>>()
        };

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);
        let next_cursor = data.last().map_or(cursor, |state_kv| {
            Some(state_kv.key_state.clone().to_string())
        });

        Ok(StatePageView {
            data,
            next_cursor,
            has_next_page,
        })
    }

    async fn get_events_by_event_handle(
        &self,
        event_handle_type: StructTagView,
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
        descending_order: Option<bool>,
        event_options: Option<EventOptions>,
    ) -> RpcResult<EventPageView> {
        let event_options = event_options.unwrap_or_default();
        let cursor = cursor.map(|v| v.0);
        let limit = limit.map(|v| v.0);
        let descending_order = descending_order.unwrap_or(true);

        // NOTE: fetch one more object to check if there is next page
        let limit_of = min(limit.unwrap_or(DEFAULT_RESULT_LIMIT), MAX_RESULT_LIMIT);
        let limit = limit_of + 1;
        let mut data = if event_options.decode {
            self.rpc_service
                .get_annotated_events_by_event_handle(
                    event_handle_type.into(),
                    cursor,
                    limit,
                    descending_order,
                )
                .await?
                .into_iter()
                .map(EventView::from)
                .collect::<Vec<_>>()
        } else {
            self.rpc_service
                .get_events_by_event_handle(
                    event_handle_type.into(),
                    cursor,
                    limit,
                    descending_order,
                )
                .await?
                .into_iter()
                .map(EventView::from)
                .collect::<Vec<_>>()
        };

        let has_next_page = (data.len() as u64) > limit_of;
        data.truncate(limit_of as usize);
        //next_cursor is the last event's event_seq
        let next_cursor = data
            .last()
            .map_or(cursor, |event| Some(event.event_id.event_seq));

        Ok(EventPageView {
            data,
            next_cursor,
            has_next_page,
        })
    }

    async fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256View>,
    ) -> RpcResult<Vec<Option<TransactionWithInfoView>>> {
        let tx_hashes: Vec<H256> = tx_hashes.iter().map(|m| (*m).into()).collect::<Vec<_>>();

        let tx_sequence_info_mapping = self
            .rpc_service
            .get_tx_sequence_info_mapping_by_hash(tx_hashes.clone())
            .await?;

        let data = self
            .aggregate_service
            .get_transaction_with_info(tx_hashes, tx_sequence_info_mapping)
            .await?
            .into_iter()
            .map(|item| item.map(TransactionWithInfoView::from))
            .collect::<Vec<_>>();

        Ok(data)
    }

    async fn get_transactions_by_order(
        &self,
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
        descending_order: Option<bool>,
    ) -> RpcResult<TransactionWithInfoPageView> {
        let last_sequencer_order = self
            .rpc_service
            .get_sequencer_order()
            .await?
            .map_or(0, |v| v.last_order);

        let limit_of = limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT);
        let descending_order = descending_order.unwrap_or(true);
        let cursor = cursor.map(|v| v.0);

        let mut tx_orders = if descending_order {
            let start = cursor.unwrap_or(last_sequencer_order + 1);
            let end = if start >= limit_of {
                start - limit_of
            } else {
                0
            };
            (end..start).rev().collect::<Vec<_>>()
        } else {
            let start = cursor.unwrap_or(0);
            let end = min(start + (limit_of + 1), last_sequencer_order + 1);
            (start..end).collect::<Vec<_>>()
        };

        // Since tx order is strictly incremental, traversing the SMT Tree can be optimized into a multi get query to improve query performance.
        let mut tx_sequence_info_mapping = self
            .rpc_service
            .get_tx_sequence_info_mapping_by_order(tx_orders.clone())
            .await?;

        let has_next_page = (tx_sequence_info_mapping.len() as u64) > limit_of;
        tx_sequence_info_mapping.truncate(limit_of as usize);
        tx_orders.truncate(limit_of as usize);
        let next_cursor = tx_sequence_info_mapping
            .last()
            .map_or(cursor, |m| m.clone().map(|v| v.tx_order));

        let mut tx_hashes = vec![];
        for item in tx_sequence_info_mapping.clone() {
            if item.is_none() {
                return Err(JsonRpcError::Custom(String::from(
                    "The tx hash corresponding to tx order does not exist",
                )));
            }
            tx_hashes.push(item.unwrap().tx_hash);
        }
        assert_eq!(tx_hashes.len(), tx_orders.len());

        let data = self
            .aggregate_service
            .get_transaction_with_info(tx_hashes, tx_sequence_info_mapping)
            .await?
            .into_iter()
            .flatten()
            .map(TransactionWithInfoView::from)
            .collect::<Vec<_>>();

        Ok(TransactionWithInfoPageView {
            data,
            next_cursor,
            has_next_page,
        })
    }

    async fn get_balance(
        &self,
        account_addr: AccountAddressView,
        coin_type: StructTagView,
    ) -> RpcResult<BalanceInfoView> {
        Ok(self
            .aggregate_service
            .get_balance(account_addr.into(), coin_type.into())
            .await
            .map(Into::into)?)
    }

    /// get account balances by AccountAddress
    async fn get_balances(
        &self,
        account_addr: AccountAddressView,
        cursor: Option<String>,
        limit: Option<StrView<usize>>,
    ) -> RpcResult<BalanceInfoPageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let cursor_of = match cursor.clone() {
            Some(key_state_str) => Some(KeyState::from_str(key_state_str.as_str())?),
            None => None,
        };

        let mut data = self
            .aggregate_service
            .get_balances(account_addr.into(), cursor_of, limit_of + 1)
            .await?;

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);

        let next_cursor = data
            .last()
            .cloned()
            .map_or(cursor, |(key, _balance_info)| key.map(|k| k.to_string()));

        Ok(BalanceInfoPageView {
            data: data
                .into_iter()
                .map(|(_, balance_info)| balance_info)
                .collect(),
            next_cursor,
            has_next_page,
        })
    }

    async fn query_transactions(
        &self,
        filter: TransactionFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<usize>>,
        descending_order: Option<bool>,
    ) -> RpcResult<TransactionWithInfoPageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let cursor = cursor.map(|v| v.0);
        let descending_order = descending_order.unwrap_or(true);

        let mut data = self
            .rpc_service
            .query_transactions(filter.into(), cursor, limit_of + 1, descending_order)
            .await?;

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);
        let next_cursor = data
            .last()
            .cloned()
            .map_or(cursor, |t| Some(t.sequence_info.tx_order));

        Ok(TransactionWithInfoPageView {
            data: data
                .into_iter()
                .map(TransactionWithInfoView::from)
                .collect::<Vec<_>>(),
            next_cursor,
            has_next_page,
        })
    }

    async fn query_events(
        &self,
        filter: EventFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerEventID>,
        limit: Option<StrView<usize>>,
        descending_order: Option<bool>,
    ) -> RpcResult<IndexerEventPageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let descending_order = descending_order.unwrap_or(true);

        let mut data = self
            .rpc_service
            .query_events(filter.into(), cursor, limit_of + 1, descending_order)
            .await?
            .into_iter()
            .map(IndexerEventView::from)
            .collect::<Vec<_>>();

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);
        let next_cursor = data
            .last()
            .cloned()
            .map_or(cursor, |e| Some(e.indexer_event_id));

        Ok(IndexerEventPageView {
            data,
            next_cursor,
            has_next_page,
        })
    }

    async fn query_global_states(
        &self,
        filter: GlobalStateFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: Option<StrView<usize>>,
        descending_order: Option<bool>,
    ) -> RpcResult<IndexerGlobalStatePageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let descending_order = descending_order.unwrap_or(true);

        // resolve multichain address
        let resolve_address = match filter.clone() {
            GlobalStateFilterView::MultiChainAddress {
                multichain_id,
                address,
            } => {
                let multi_chain_address = MultiChainAddress::try_from_str_with_multichain_id(
                    RoochMultiChainID::try_from(multichain_id)?,
                    address.as_str(),
                )?;
                self.rpc_service
                    .resolve_address(multi_chain_address)
                    .await?
            }
            _ => AccountAddress::ZERO,
        };
        let global_state_filter =
            GlobalStateFilterView::into_global_state_filter(filter, resolve_address);
        let mut data = self
            .rpc_service
            .query_global_states(global_state_filter, cursor, limit_of + 1, descending_order)
            .await?
            .into_iter()
            .map(IndexerGlobalStateView::try_new_from_global_state)
            .collect::<Result<Vec<_>>>()?;

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);
        let next_cursor = data.last().cloned().map_or(cursor, |t| {
            Some(IndexerStateID::new(t.tx_order, t.state_index))
        });

        Ok(IndexerGlobalStatePageView {
            data,
            next_cursor,
            has_next_page,
        })
    }

    async fn query_table_states(
        &self,
        filter: TableStateFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: Option<StrView<usize>>,
        descending_order: Option<bool>,
    ) -> RpcResult<IndexerTableStatePageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let descending_order = descending_order.unwrap_or(true);

        let mut data = self
            .rpc_service
            .query_table_states(filter.into(), cursor, limit_of + 1, descending_order)
            .await?
            .into_iter()
            .map(IndexerTableStateView::try_new_from_table_state)
            .collect::<Result<Vec<_>>>()?;

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);
        let next_cursor = data.last().cloned().map_or(cursor, |t| {
            Some(IndexerStateID::new(t.tx_order, t.state_index))
        });

        Ok(IndexerTableStatePageView {
            data,
            next_cursor,
            has_next_page,
        })
    }

    async fn sync_states(
        &self,
        filter: Option<StateSyncFilterView>,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: Option<StrView<usize>>,
        descending_order: Option<bool>,
    ) -> RpcResult<IndexerTableChangeSetPageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        // Sync from asc by default
        let descending_order = descending_order.unwrap_or(false);

        let mut data = self
            .rpc_service
            .sync_states(
                filter.map(Into::into),
                cursor,
                limit_of + 1,
                descending_order,
            )
            .await?
            .into_iter()
            .map(IndexerTableChangeSetView::from)
            .collect::<Vec<_>>();

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);
        let next_cursor = data.last().cloned().map_or(cursor, |t| {
            Some(IndexerStateID::new(t.tx_order, t.state_index))
        });

        Ok(IndexerTableChangeSetPageView {
            data,
            next_cursor,
            has_next_page,
        })
    }
}

impl RoochRpcModule for RoochServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
