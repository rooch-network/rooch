// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::aggregate_service::AggregateService;
use crate::service::rpc_service::RpcService;
use anyhow::Result;
use jsonrpsee::core::SubscriptionResult;
use jsonrpsee::{core::async_trait, PendingSubscriptionSink, RpcModule};
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
};
use moveos_types::moveos_std::event::EventHandle;
use moveos_types::{
    access_path::AccessPath,
    h256::H256,
    move_std::string::MoveString,
    moveos_std::{move_module::MoveModule, object::ObjectID},
    state::{AnnotatedState, FieldKey},
};
use rooch_rpc_api::api::MAX_INTERNAL_LIMIT_USIZE;
use rooch_rpc_api::jsonrpc_types::field_view::FieldFilterView;
use rooch_rpc_api::jsonrpc_types::{
    account_view::BalanceInfoView,
    event_view::{EventFilterView, EventView, IndexerEventIDView, IndexerEventView},
    transaction_view::{TransactionFilterView, TransactionWithInfoView},
    AccessPathView, BalanceInfoPageView, DryRunTransactionResponseView, EventOptions,
    EventPageView, ExecuteTransactionResponseView, FieldPageView, FunctionCallView, H256View,
    IndexerEventPageView, IndexerObjectStatePageView, IndexerStateIDView, ModuleABIView,
    ObjectIDVecView, ObjectIDView, ObjectStateFilterView, ObjectStateView, QueryOptions,
    RawTransactionOutputView, RoochAddressView, StateChangeSetPageView,
    StateChangeSetWithTxOrderView, StateKVView, StateOptions, StatePageView, StrView,
    StructTagView, SyncStateFilterView, TransactionWithInfoPageView, TxOptions, UnitedAddressView,
};
use rooch_rpc_api::jsonrpc_types::{
    repair_view::{RepairIndexerParamsView, RepairIndexerTypeView},
    Status,
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
use rooch_types::indexer::state::{IndexerStateID, ObjectStateType};
use rooch_types::transaction::{RoochTransaction, RoochTransactionData, TransactionWithInfo};
use std::cmp::{max, min};
use std::str::FromStr;
use tracing::{debug, info};

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
        let bitcoin_network = self.rpc_service.get_bitcoin_network();
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
        let chain_id = self.rpc_service.get_chain_id();
        Ok(StrView(chain_id))
    }

    async fn send_raw_transaction(&self, payload: BytesView) -> RpcResult<H256View> {
        debug!("send_raw_transaction payload: {:?}", payload);
        let mut tx = bcs::from_bytes::<RoochTransaction>(&payload.0)?;
        info!(
            "send_raw_transaction tx sender:{:?}, hash:{:?}",
            tx.sender(),
            tx.tx_hash()
        );

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
            let mut txn_resp_view = ExecuteTransactionResponseView::from(tx_response.clone());
            if tx_options.decode {
                let event_ids = tx_response
                    .output
                    .events
                    .iter()
                    .map(|e| e.event_id.clone())
                    .collect();
                let annotated_events = self
                    .rpc_service
                    .get_annotated_events_by_event_ids(event_ids)
                    .await?;
                let event_views = annotated_events
                    .into_iter()
                    .map(|event| event.map(EventView::from))
                    .collect::<Vec<Option<_>>>();
                debug_assert!(
                    txn_resp_view.output.is_some()
                        && event_views.len() == txn_resp_view.output.as_ref().unwrap().events.len(),
                    "event_views length should be equal to txn_resp_view.output.events length"
                );
                let output_view = txn_resp_view.output.clone().map(|mut output| {
                    output.events.iter_mut().zip(event_views).for_each(
                        |(event, event_view_opt)| {
                            if let Some(decoded_event_view) = event_view_opt {
                                event.decoded_event_data = decoded_event_view.decoded_event_data;
                            }
                        },
                    );
                    output
                });
                txn_resp_view.output = output_view;
                txn_resp_view
            } else {
                txn_resp_view
            }
        } else {
            ExecuteTransactionResponseView::new_without_output(tx_response)
        };
        Ok(result)
    }

    async fn dry_run(&self, payload: BytesView) -> RpcResult<DryRunTransactionResponseView> {
        let tx = bcs::from_bytes::<RoochTransactionData>(&payload.0)?;
        let tx_hash = tx.tx_hash();
        let tx_result = self.rpc_service.dry_run_tx(tx).await?;
        let raw_output = tx_result.raw_output;
        let raw_output_view = RawTransactionOutputView {
            tx_hash: tx_hash.into(),
            state_root: raw_output.changeset.state_root.into(),
            status: raw_output.status.into(),
            gas_used: raw_output.gas_used.into(),
            is_upgrade: raw_output.is_upgrade,
        };
        let tx_response = DryRunTransactionResponseView {
            raw_output: raw_output_view,
            vm_error_info: tx_result.vm_error_info.unwrap_or_default(),
        };
        Ok(tx_response)
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
        access_path.0.validate_max_object_ids()?;
        let state_option = state_option.unwrap_or_default();
        let show_display =
            state_option.show_display && (access_path.0.is_object() || access_path.0.is_resource());

        let state_root = state_option.state_root.map(|h256_view| h256_view.0);

        let state_views = if state_option.decode || show_display {
            let states = self
                .rpc_service
                .get_annotated_states(access_path.into(), state_root)
                .await?;

            if show_display {
                let valid_states = states.iter().filter_map(|s| s.as_ref()).collect::<Vec<_>>();
                let mut valid_display_field_views = self
                    .rpc_service
                    .get_display_fields_and_render(valid_states.as_slice(), state_root)
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
                .get_states(access_path.into(), state_root)
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
        limit: Option<StrView<u64>>,
        state_option: Option<StateOptions>,
    ) -> RpcResult<StatePageView> {
        access_path.0.validate_max_object_ids()?;
        let state_option = state_option.unwrap_or_default();
        let show_display =
            state_option.show_display && (access_path.0.is_object() || access_path.0.is_resource());

        let state_root = state_option.state_root.map(|h256_view| h256_view.0);

        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let cursor_of = match cursor.clone() {
            Some(key_state_str) => Some(FieldKey::from_str(key_state_str.as_str())?),
            None => None,
        };
        let mut data: Vec<StateKVView> = if state_option.decode || show_display {
            let (key_states, states): (Vec<FieldKey>, Vec<AnnotatedState>) = self
                .rpc_service
                .list_annotated_states(state_root, access_path.into(), cursor_of, limit_of + 1)
                .await?
                .into_iter()
                .unzip();
            let state_refs: Vec<&AnnotatedState> = states.iter().collect();
            if show_display {
                let display_field_views = self
                    .rpc_service
                    .get_display_fields_and_render(state_refs.as_slice(), state_root)
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
                .list_states(state_root, access_path.into(), cursor_of, limit_of + 1)
                .await?
                .into_iter()
                .map(|(key, state)| StateKVView::new(key.into(), state.into()))
                .collect::<Vec<_>>()
        };

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);
        let next_cursor = data.last().map_or(cursor, |state_kv| {
            Some(state_kv.field_key.clone().to_string())
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

        let mut objects_view = if decode || show_display {
            let states: Vec<Option<AnnotatedState>> = self
                .rpc_service
                .get_annotated_states(access_path, None)
                .await?;

            let mut valid_display_field_views = if show_display {
                let valid_states = states.iter().filter_map(|s| s.as_ref()).collect::<Vec<_>>();
                self.rpc_service
                    .get_display_fields_and_render(valid_states.as_slice(), None)
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
                .get_states(access_path, None)
                .await?
                .into_iter()
                .map(|s| s.map(Into::into))
                .collect()
        };

        self.rpc_service
            .fill_bitcoin_addresses(
                objects_view
                    .iter_mut()
                    .filter_map(|state_opt| state_opt.as_mut().map(|state| &mut state.metadata))
                    .collect(),
            )
            .await?;

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
        let event_handle_id = EventHandle::derive_event_handle_id(&event_handle_type.into());
        self.get_events_by_event_handle_v2(
            event_handle_id.into(),
            cursor,
            limit,
            descending_order,
            event_options,
        )
        .await
    }

    async fn get_events_by_event_handle_v2(
        &self,
        event_handle_id: ObjectIDView,
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
                    event_handle_id.0,
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
                .get_events_by_event_handle(event_handle_id.0, cursor, limit, descending_order)
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
            .map_or(cursor, |event| Some(event.event_id.event_seq.0));

        Ok(EventPageView {
            data,
            next_cursor: next_cursor.map(StrView),
            has_next_page,
        })
    }

    async fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256View>,
    ) -> RpcResult<Vec<Option<TransactionWithInfoView>>> {
        let tx_hashes: Vec<H256> = tx_hashes.iter().map(|m| (*m).into()).collect::<Vec<_>>();

        let bitcoin_network = self.rpc_service.get_bitcoin_network();
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
            let start = cursor.map(|s| s + 1).unwrap_or(0);
            let start_plus = start
                .checked_add(limit_of + 1)
                .ok_or(RpcError::UnexpectedError(
                    "cursor value is overflow".to_string(),
                ))?;
            let end = min(start_plus, last_sequencer_order + 1);

            (start..end).collect::<Vec<_>>()
        };

        let tx_hashes = self.rpc_service.get_tx_hashes(tx_orders.clone()).await?;

        let mut hash_order_pair = tx_hashes
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
            next_cursor: next_cursor.map(StrView),
            has_next_page,
        })
    }

    async fn get_balance(
        &self,
        account_addr: UnitedAddressView,
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
        account_addr: UnitedAddressView,
        cursor: Option<IndexerStateIDView>,
        limit: Option<StrView<u64>>,
    ) -> RpcResult<BalanceInfoPageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let cursor: Option<IndexerStateID> = cursor.map(Into::into);
        let mut data = self
            .aggregate_service
            .get_balances(account_addr.into(), cursor.clone(), limit_of + 1)
            .await?;

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);

        let next_cursor = data
            .last()
            .cloned()
            .map_or(cursor.clone(), |(key, _balance_info)| key);

        Ok(BalanceInfoPageView {
            data: data
                .into_iter()
                .map(|(_, balance_info)| balance_info)
                .collect(),
            next_cursor: next_cursor.map(Into::into),
            has_next_page,
        })
    }

    async fn get_module_abi(
        &self,
        module_addr: RoochAddressView,
        module_name: String,
    ) -> RpcResult<Option<ModuleABIView>> {
        let module_id = ModuleId::new(
            AccountAddress::from(module_addr.0),
            Identifier::new(module_name)?,
        );
        let access_path = AccessPath::module(&module_id);
        let module = self
            .rpc_service
            .get_states(access_path, None)
            .await?
            .pop()
            .flatten();

        Ok(match module {
            Some(m) => {
                let move_module = m.value_as_df::<MoveString, MoveModule>()?.value;
                Some(ModuleABIView::try_parse_from_module_bytes(
                    &move_module.byte_codes,
                )?)
            }
            None => None,
        })
    }

    async fn query_transactions(
        &self,
        filter: TransactionFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
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
            next_cursor: next_cursor.map(StrView),
            has_next_page,
        })
    }

    async fn query_events(
        &self,
        filter: EventFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerEventIDView>,
        limit: Option<StrView<u64>>,
        query_option: Option<QueryOptions>,
    ) -> RpcResult<IndexerEventPageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let query_option = query_option.unwrap_or_default();
        let descending_order = query_option.descending;

        let mut data = if query_option.decode {
            self.rpc_service
                .query_annotated_events(
                    filter.into(),
                    cursor.map(Into::into),
                    limit_of + 1,
                    descending_order,
                )
                .await?
                .into_iter()
                .map(IndexerEventView::from)
                .collect::<Vec<_>>()
        } else {
            self.rpc_service
                .query_events(
                    filter.into(),
                    cursor.map(Into::into),
                    limit_of + 1,
                    descending_order,
                )
                .await?
                .into_iter()
                .map(IndexerEventView::from)
                .collect::<Vec<_>>()
        };

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
        cursor: Option<IndexerStateIDView>,
        limit: Option<StrView<u64>>,
        query_option: Option<QueryOptions>,
    ) -> RpcResult<IndexerObjectStatePageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let query_option = query_option.unwrap_or_default();
        let descending_order = query_option.descending;

        let global_state_filter =
            ObjectStateFilterView::try_into_object_state_filter(filter, query_option.clone())?;
        let mut object_states = self
            .rpc_service
            .query_object_states(
                global_state_filter,
                cursor.map(Into::into),
                limit_of + 1,
                descending_order,
                query_option.decode,
                query_option.show_display,
                ObjectStateType::ObjectState,
            )
            .await?;

        let has_next_page = object_states.len() > limit_of;
        object_states.truncate(limit_of);

        let next_cursor = object_states
            .last()
            .cloned()
            .map_or(cursor, |t| Some(t.indexer_id));

        Ok(IndexerObjectStatePageView {
            data: object_states,
            next_cursor,
            has_next_page,
        })
    }

    async fn query_fields(
        &self,
        filter: FieldFilterView,
        page: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
        query_option: Option<QueryOptions>,
    ) -> RpcResult<FieldPageView> {
        let page_of = max(page.map(Into::into).unwrap_or(1), 1u64);
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let query_option = query_option.unwrap_or_default();
        let descending_order = query_option.descending;
        let decode = query_option.decode;

        let (fields, mut fields_view) = self
            .rpc_service
            .query_fields(
                filter.into(),
                page_of,
                limit_of + 1,
                descending_order,
                decode,
            )
            .await?;

        let has_next_page = fields.len() > limit_of;
        // Solve the pagation consistency problem after indexer data filtering
        if fields_view.len() >= fields.len() {
            fields_view.truncate(limit_of);
        }

        let next_page_check = if has_next_page {
            page_of.checked_add(1).ok_or(RpcError::UnexpectedError(
                "next page value is overflow".to_string(),
            ))?
        } else {
            page_of
        };
        let next_cursor = Some(StrView(next_page_check));

        Ok(FieldPageView {
            data: fields_view,
            next_cursor,
            has_next_page,
        })
    }

    async fn repair_indexer(
        &self,
        repair_type: RepairIndexerTypeView,
        repair_params: RepairIndexerParamsView,
    ) -> RpcResult<()> {
        self.rpc_service
            .repair_indexer(repair_type.0, repair_params.into())
            .await?;
        Ok(())
    }

    async fn sync_states(
        &self,
        filter: SyncStateFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
        query_option: Option<QueryOptions>,
    ) -> RpcResult<StateChangeSetPageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        ) as u64;
        let cursor_of = cursor.map(|v| v.0);
        // Sync from asc by default
        let descending_order = query_option.map(|v| v.descending).unwrap_or(false);

        let last_sequencer_order = self.rpc_service.get_sequencer_order().await?;
        let tx_orders = if descending_order {
            let start = cursor_of.unwrap_or(last_sequencer_order + 1);
            let end = if start >= (limit_of + 1) {
                start - (limit_of + 1)
            } else {
                0
            };

            (end..start).rev().collect::<Vec<_>>()
        } else {
            let start = cursor_of.map(|s| s + 1).unwrap_or(0);
            let end_check = start
                .checked_add(limit_of + 1)
                .ok_or(RpcError::UnexpectedError(
                    "cursor value is overflow".to_string(),
                ))?;
            let end = min(end_check, last_sequencer_order + 1);

            (start..end).collect::<Vec<_>>()
        };

        let mut data = self
            .rpc_service
            .sync_states(tx_orders, filter.into())
            .await?
            .into_iter()
            .map(StateChangeSetWithTxOrderView::from)
            .collect::<Vec<_>>();

        let has_next_page = data.len() > limit_of as usize;
        data.truncate(limit_of as usize);
        let next_cursor = data.last().cloned().map_or(cursor, |t| Some(t.tx_order));

        Ok(StateChangeSetPageView {
            data,
            next_cursor,
            has_next_page,
        })
    }

    async fn status(&self) -> RpcResult<Status> {
        let status = self.rpc_service.status().await?;
        Ok(status)
    }

    async fn check_change_set(
        &self,
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
        query_option: Option<QueryOptions>,
    ) -> RpcResult<Vec<u64>> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(MAX_INTERNAL_LIMIT_USIZE),
            MAX_INTERNAL_LIMIT_USIZE,
        ) as u64;
        let cursor_of = cursor.map(|v| v.0);
        // Sync from asc by default
        let descending_order = query_option.map(|v| v.descending).unwrap_or(false);

        let last_sequencer_order = self.rpc_service.get_sequencer_order().await?;
        let tx_orders = if descending_order {
            let start = cursor_of.unwrap_or(last_sequencer_order + 1);
            let end = if start >= (limit_of + 1) {
                start - (limit_of + 1)
            } else {
                0
            };

            (end..start).rev().collect::<Vec<_>>()
        } else {
            let start = cursor_of.map(|s| s + 1).unwrap_or(0);
            let end_check = start
                .checked_add(limit_of + 1)
                .ok_or(RpcError::UnexpectedError(
                    "cursor value is overflow".to_string(),
                ))?;
            let end = min(end_check, last_sequencer_order + 1);

            (start..end).collect::<Vec<_>>()
        };

        let mut data = self.rpc_service.check_state_change_sets(tx_orders).await?;
        data.truncate(limit_of as usize);

        Ok(data)
    }

    fn subscribe_events(
        &self,
        sink: PendingSubscriptionSink,
        filter: EventFilterView,
    ) -> SubscriptionResult {
        self.rpc_service.subscribe_events(sink, filter)?;
        Ok(())
    }

    fn subscribe_transactions(
        &self,
        sink: PendingSubscriptionSink,
        filter: TransactionFilterView,
    ) -> SubscriptionResult {
        self.rpc_service.subscribe_transactions(sink, filter)?;
        Ok(())
    }
}

impl RoochRpcModule for RoochServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
