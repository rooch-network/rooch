// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::aggregate_service::AggregateService;
use crate::service::rpc_service::RpcService;
use anyhow::Result;
use jsonrpsee::{core::async_trait, RpcModule};
use moveos_types::{
    access_path::AccessPath,
    h256::H256,
    moveos_std::{
        display::{get_object_display_id, get_resource_display_id, RawDisplay},
        object::{ObjectEntity, ObjectID},
    },
    state::{AnnotatedState, FieldKey},
};
use rooch_rpc_api::jsonrpc_types::{
    account_view::BalanceInfoView,
    event_view::{EventFilterView, EventView, IndexerEventView},
    transaction_view::{TransactionFilterView, TransactionWithInfoView},
    AccessPathView, AnnotatedMoveStructView, BalanceInfoPageView, DisplayFieldsView, EventOptions,
    EventPageView, ExecuteTransactionResponseView, FunctionCallView, H256View,
    IndexerEventPageView, IndexerObjectStatePageView, IndexerObjectStateView, ObjectIDVecView,
    ObjectStateFilterView, ObjectStateView, QueryOptions, RoochOrBitcoinAddressView, StateKVView,
    StateOptions, StatePageView, StrView, StructTagView, TransactionWithInfoPageView, TxOptions,
};
use rooch_rpc_api::{
    api::rooch_api::RoochAPIServer,
    api::DEFAULT_RESULT_LIMIT,
    api::{RoochRpcModule, DEFAULT_RESULT_LIMIT_USIZE},
    api::{MAX_RESULT_LIMIT, MAX_RESULT_LIMIT_USIZE},
    jsonrpc_types::AnnotatedFunctionResultView,
    jsonrpc_types::BytesView,
    RpcError, RpcResult,
};
use rooch_types::indexer::event::IndexerEventID;
use rooch_types::indexer::state::IndexerStateID;
use rooch_types::transaction::{RoochTransaction, TransactionWithInfo};
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
                let value_struct_tag = s.metadata.value_struct_tag();
                display_ids.push(get_object_display_id(value_struct_tag.clone().into()));
                true
            } else if let Some(tag) = s.metadata.get_resource_struct_tag() {
                display_ids.push(get_resource_display_id(tag.clone().into()));
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
                    .map(|s| s.into_object_uncheck::<RawDisplay>())
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
                display_fields.pop().unwrap().map(|obj| {
                    DisplayFieldsView::new(obj.value.render(
                        &move_resource_viewer::AnnotatedMoveValue::Struct(
                            annotated_s.decoded_value.clone(),
                        ),
                    ))
                })
            } else {
                None
            });
        }
        Ok(display_field_views)
    }

    async fn transactions_to_view(
        &self,
        data: Vec<TransactionWithInfo>,
    ) -> Result<Vec<TransactionWithInfoView>> {
        let rooch_addresses = data
            .iter()
            .filter_map(|tx| tx.transaction.sender())
            .collect::<Vec<_>>();
        let address_mapping = self
            .rpc_service
            .get_bitcoin_addresses(rooch_addresses)
            .await?;
        let bitcoin_network = self.rpc_service.get_bitcoin_network().await?;
        let data = data
            .into_iter()
            .map(|tx| {
                let sender_bitcoin_address = match tx.transaction.sender() {
                    Some(rooch_address) => address_mapping
                        .get(&rooch_address)
                        .map(|addr| addr.clone().map(|a| a.format(bitcoin_network))),
                    None => None,
                }
                .flatten()
                .transpose()?;
                Ok(TransactionWithInfoView::new_from_transaction_with_info(
                    tx,
                    sender_bitcoin_address,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(data)
    }
}

#[async_trait]
impl RoochAPIServer for RoochServer {
    async fn get_chain_id(&self) -> RpcResult<StrView<u64>> {
        let chain_id = self.rpc_service.get_chain_id().await?;
        Ok(StrView(chain_id))
    }

    async fn send_raw_transaction(&self, payload: BytesView) -> RpcResult<H256View> {
        info!("send_raw_transaction payload: {:?}", payload);
        let mut tx = bcs::from_bytes::<RoochTransaction>(&payload.0)?;
        info!("send_raw_transaction tx: {:?}", tx);

        let hash = tx.tx_hash();
        self.rpc_service.queue_tx(tx).await?;
        Ok(hash.into())
    }

    async fn execute_raw_transaction(
        &self,
        payload: BytesView,
        tx_options: Option<TxOptions>,
    ) -> RpcResult<ExecuteTransactionResponseView> {
        let tx_options = tx_options.unwrap_or_default();
        let tx = bcs::from_bytes::<RoochTransaction>(&payload.0)?;
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
    ) -> RpcResult<Vec<Option<ObjectStateView>>> {
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
                            ObjectStateView::from(annotated_s).with_display_fields(display_view)
                        })
                    })
                    .collect()
            } else {
                states
                    .into_iter()
                    .map(|s| s.map(ObjectStateView::from))
                    .collect()
            }
        } else {
            self.rpc_service
                .get_states(access_path.into())
                .await?
                .into_iter()
                .map(|s| s.map(ObjectStateView::from))
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
            Some(key_state_str) => Some(FieldKey::from_str(key_state_str.as_str())?),
            None => None,
        };
        let mut data: Vec<StateKVView> = if state_option.decode || show_display {
            let is_object = access_path.0.is_object();
            let (key_states, states): (Vec<FieldKey>, Vec<AnnotatedState>) = self
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
                    .map(|((key, state), display_field_view)| {
                        StateKVView::new(
                            key.into(),
                            ObjectStateView::from(state).with_display_fields(display_field_view),
                        )
                    })
                    .collect::<Vec<_>>()
            } else {
                key_states
                    .into_iter()
                    .zip(states)
                    .map(|(key, state)| StateKVView::new(key.into(), state.into()))
                    .collect::<Vec<_>>()
            }
        } else {
            self.rpc_service
                .list_states(access_path.into(), cursor_of, limit_of + 1)
                .await?
                .into_iter()
                .map(|(key, state)| StateKVView::new(key.into(), state.into()))
                .collect::<Vec<_>>()
        };

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);
        let next_cursor = data.last().map_or(cursor, |state_kv| {
            Some(state_kv.key_hex.clone().to_string())
        });

        Ok(StatePageView {
            data,
            next_cursor,
            has_next_page,
        })
    }

    async fn get_object_states(
        &self,
        object_ids: ObjectIDVecView,
        state_option: Option<StateOptions>,
    ) -> RpcResult<Vec<Option<ObjectStateView>>> {
        let object_ids: Vec<ObjectID> = object_ids.into();
        let access_path = AccessPath::objects(object_ids);
        let state_option = state_option.unwrap_or_default();
        let decode = state_option.decode;
        let show_display = state_option.show_display;

        let objects_view = if decode || show_display {
            let states: Vec<Option<AnnotatedState>> =
                self.rpc_service.get_annotated_states(access_path).await?;

            let mut valid_display_field_views = if show_display {
                let valid_states = states.iter().filter_map(|s| s.as_ref()).collect::<Vec<_>>();
                self.get_display_fields_and_render(valid_states, true)
                    .await?
            } else {
                vec![]
            };

            let objects_view = states
                .into_iter()
                .map(|option_annotated_s| {
                    option_annotated_s.map(|annotated_s| ObjectStateView::new(annotated_s, decode))
                })
                .collect::<Vec<_>>();

            if show_display {
                valid_display_field_views.reverse();
                objects_view
                    .into_iter()
                    .map(|option_state_view| {
                        option_state_view.map(|sview| {
                            debug_assert!(
                                !valid_display_field_views.is_empty(),
                                "display fields should not be empty"
                            );
                            let display_view = valid_display_field_views.pop().unwrap();
                            sview.with_display_fields(display_view)
                        })
                    })
                    .collect()
            } else {
                objects_view
            }
        } else {
            self.rpc_service
                .get_states(access_path)
                .await?
                .into_iter()
                .map(|s| s.map(Into::into))
                .collect()
        };

        // Get owner_bitcoin_address
        let addresses = objects_view
            .iter()
            .filter_map(|o| o.as_ref().map(|s| s.owner.into()))
            .collect::<Vec<_>>();
        let btc_network = self.rpc_service.get_bitcoin_network().await?;
        let address_mapping = self.rpc_service.get_bitcoin_addresses(addresses).await?;

        let objects_view = objects_view
            .into_iter()
            .map(|o| {
                o.map(|s| {
                    let rooch_address = s.owner.into();
                    let bitcoin_address = address_mapping
                        .get(&rooch_address)
                        .expect("should exist.")
                        .clone()
                        .map(|a| a.format(btc_network))
                        .transpose()?;
                    Ok(s.with_owner_bitcoin_address(bitcoin_address))
                })
                .transpose()
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(objects_view)
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

        let bitcoin_network = self.rpc_service.get_bitcoin_network().await?;
        let data = self
            .aggregate_service
            .get_transaction_with_info(tx_hashes)
            .await?;

        let rooch_addresses = data
            .iter()
            .filter_map(|tx| tx.as_ref().and_then(|tx| tx.transaction.sender()))
            .collect::<Vec<_>>();
        let address_mapping = self
            .rpc_service
            .get_bitcoin_addresses(rooch_addresses)
            .await?;

        let data = data
            .into_iter()
            .map(|item| {
                item.map(|tx| {
                    let sender_bitcoin_address = match tx.transaction.sender() {
                        Some(rooch_address) => address_mapping
                            .get(&rooch_address)
                            .map(|addr| addr.clone().map(|a| a.format(bitcoin_network))),
                        None => None,
                    }
                    .flatten()
                    .transpose()?;
                    Ok(TransactionWithInfoView::new_from_transaction_with_info(
                        tx,
                        sender_bitcoin_address,
                    ))
                })
                .transpose()
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(data)
    }

    async fn get_transactions_by_order(
        &self,
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
        descending_order: Option<bool>,
    ) -> RpcResult<TransactionWithInfoPageView> {
        let last_sequencer_order = self.rpc_service.get_sequencer_order().await?;

        let limit_of = min(
            limit
                .map(Into::into)
                .unwrap_or(DEFAULT_RESULT_LIMIT_USIZE as u64),
            MAX_RESULT_LIMIT_USIZE as u64,
        );

        let descending_order = descending_order.unwrap_or(true);
        let cursor = cursor.map(|v| v.0);

        let tx_orders = if descending_order {
            let start = cursor.unwrap_or(last_sequencer_order + 1);
            let end = if start >= limit_of {
                start - limit_of
            } else {
                0
            };

            (end..start).rev().collect::<Vec<_>>()
        } else {
            let start = cursor.unwrap_or(0);
            let start_plus = start
                .checked_add(limit_of + 1)
                .ok_or(RpcError::UnexpectedError(
                    "cursor value is overflow".to_string(),
                ))?;
            let end = min(start_plus, last_sequencer_order + 1);

            (start..end).collect::<Vec<_>>()
        };

        let tx_hashs = self.rpc_service.get_tx_hashs(tx_orders.clone()).await?;

        let mut hash_order_pair = tx_hashs
            .into_iter()
            .zip(tx_orders)
            .filter_map(|(h, o)| h.map(|h| (h, o)))
            .collect::<Vec<_>>();

        let has_next_page = (hash_order_pair.len() as u64) > limit_of;
        hash_order_pair.truncate(limit_of as usize);

        let next_cursor = hash_order_pair.last().map_or(cursor, |(_h, o)| Some(*o));

        let data = self
            .aggregate_service
            .get_transaction_with_info(
                hash_order_pair
                    .into_iter()
                    .map(|(h, _o)| h)
                    .collect::<Vec<_>>(),
            )
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        let data = self.transactions_to_view(data).await?;

        Ok(TransactionWithInfoPageView {
            data,
            next_cursor,
            has_next_page,
        })
    }

    async fn get_balance(
        &self,
        account_addr: RoochOrBitcoinAddressView,
        coin_type: StructTagView,
    ) -> RpcResult<BalanceInfoView> {
        Ok(self
            .aggregate_service
            .get_balance(account_addr.into(), coin_type.into())
            .await
            .map(Into::into)?)
    }

    /// get account balances by RoochAddress
    async fn get_balances(
        &self,
        account_addr: RoochOrBitcoinAddressView,
        cursor: Option<IndexerStateID>,
        limit: Option<StrView<usize>>,
    ) -> RpcResult<BalanceInfoPageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let mut data = self
            .aggregate_service
            .get_balances(account_addr.into(), cursor, limit_of + 1)
            .await?;

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);

        let next_cursor = data
            .last()
            .cloned()
            .map_or(cursor, |(key, _balance_info)| key);

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
        query_option: Option<QueryOptions>,
    ) -> RpcResult<TransactionWithInfoPageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let cursor = cursor.map(|v| v.0);
        let query_option = query_option.unwrap_or_default();
        let descending_order = query_option.descending;

        let txs = self
            .rpc_service
            .query_transactions(filter.into(), cursor, limit_of + 1, descending_order)
            .await?;

        let mut data = self
            .aggregate_service
            .build_transaction_with_infos(txs)
            .await?;

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);

        let data = self.transactions_to_view(data).await?;

        let next_cursor = data
            .last()
            .cloned()
            .map_or(cursor, |t| Some(t.transaction.sequence_info.tx_order.0));

        Ok(TransactionWithInfoPageView {
            data,
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
        query_option: Option<QueryOptions>,
    ) -> RpcResult<IndexerEventPageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let query_option = query_option.unwrap_or_default();
        let descending_order = query_option.descending;

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

    async fn query_object_states(
        &self,
        filter: ObjectStateFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: Option<StrView<usize>>,
        query_option: Option<QueryOptions>,
    ) -> RpcResult<IndexerObjectStatePageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let query_option = query_option.unwrap_or_default();
        let descending_order = query_option.descending;
        let decode = query_option.decode;

        let global_state_filter = ObjectStateFilterView::try_into_object_state_filter(filter)?;
        let states = self
            .rpc_service
            .query_object_states(global_state_filter, cursor, limit_of + 1, descending_order)
            .await?;

        let object_ids = states
            .iter()
            .map(|m| m.object_id.clone())
            .collect::<Vec<_>>();
        let access_path = AccessPath::objects(object_ids.clone());
        let annotated_states = self
            .rpc_service
            .get_annotated_states(access_path)
            .await?
            .into_iter()
            .map(|s| s.expect("object should exist!")) // TODO: is statedb always synced with indexer?
            .collect::<Vec<_>>();

        let annotated_states_with_display = if query_option.show_display {
            let valid_states = annotated_states.iter().collect::<Vec<_>>();
            let valid_display_field_views = self
                .get_display_fields_and_render(valid_states, true)
                .await?;
            annotated_states
                .into_iter()
                .zip(valid_display_field_views)
                .collect::<Vec<_>>()
        } else {
            annotated_states
                .into_iter()
                .map(|s| (s, None))
                .collect::<Vec<_>>()
        };

        let network = self.rpc_service.get_bitcoin_network().await?;
        let rooch_addresses = states.iter().map(|s| s.owner).collect::<Vec<_>>();
        let bitcoin_addresses = self
            .rpc_service
            .get_bitcoin_addresses(rooch_addresses)
            .await?
            .into_values()
            .map(|btc_addr| btc_addr.map(|addr| addr.format(network)).transpose())
            .collect::<Result<Vec<Option<String>>>>()?;

        let mut data = annotated_states_with_display
            .into_iter()
            .zip(states)
            .zip(bitcoin_addresses)
            .map(
                |(((annotated_state, display_fields), state), bitcoin_address)| {
                    let decoded_value = if decode {
                        Some(AnnotatedMoveStructView::from(annotated_state.decoded_value))
                    } else {
                        None
                    };
                    IndexerObjectStateView::new_from_object_state(
                        state,
                        annotated_state.value,
                        bitcoin_address,
                        decoded_value,
                        display_fields,
                    )
                },
            )
            .collect::<Vec<_>>();

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);

        let next_cursor = data.last().cloned().map_or(cursor, |t| {
            Some(IndexerStateID::new(t.tx_order, t.state_index))
        });

        Ok(IndexerObjectStatePageView {
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
