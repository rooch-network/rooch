// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use bitcoin_client::proxy::BitcoinClientProxy;
use bitcoincore_rpc::bitcoin::Txid;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::{ModuleId, StructTag};
use moveos_types::access_path::AccessPath;
use moveos_types::function_return_value::AnnotatedFunctionResult;
use moveos_types::h256::H256;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::move_types::type_tag_match;
use moveos_types::moveos_std::display::{get_object_display_id, RawDisplay};
use moveos_types::moveos_std::event::{AnnotatedEvent, Event, EventID};
use moveos_types::moveos_std::object::{ObjectID, MAX_OBJECT_IDS_PER_QUERY};
use moveos_types::state::{AnnotatedState, FieldKey, ObjectState, StateChangeSet};
use moveos_types::state_resolver::{AnnotatedStateKV, StateKV};
use moveos_types::transaction::{FunctionCall, TransactionExecutionInfo};
use rooch_da::proxy::DAServerProxy;
use rooch_executor::actor::messages::DryRunTransactionResult;
use rooch_executor::proxy::ExecutorProxy;
use rooch_indexer::proxy::IndexerProxy;
use rooch_pipeline_processor::proxy::PipelineProcessorProxy;
use rooch_rpc_api::jsonrpc_types::{
    BitcoinStatus, DisplayFieldsView, IndexerObjectStateView, ObjectMetaView, RoochStatus, Status,
};
use rooch_sequencer::proxy::SequencerProxy;
use rooch_types::address::{BitcoinAddress, RoochAddress};
use rooch_types::bitcoin::pending_block::PendingBlockModule;
use rooch_types::bitcoin::BitcoinModule;
use rooch_types::framework::address_mapping::RoochToBitcoinAddressMapping;
use rooch_types::indexer::event::{
    AnnotatedIndexerEvent, EventFilter, IndexerEvent, IndexerEventID,
};
use rooch_types::indexer::state::{
    IndexerObjectState, IndexerStateID, ObjectStateFilter, ObjectStateType, INSCRIPTION_TYPE_TAG,
    UTXO_TYPE_TAG,
};
use rooch_types::indexer::transaction::{IndexerTransaction, TransactionFilter};
use rooch_types::repair::{RepairIndexerParams, RepairIndexerType};
use rooch_types::state::{StateChangeSetWithTxOrder, SyncStateFilter};
use rooch_types::transaction::{
    ExecuteTransactionResponse, LedgerTransaction, RoochTransaction, RoochTransactionData,
};
use std::collections::{BTreeMap, HashMap};

/// RpcService is the implementation of the RPC service.
/// It is the glue between the RPC server(EthAPIServer,RoochApiServer) and the rooch's actors.
/// The RpcService encapsulates the logic of the functions, and the RPC server handle the response format.
#[derive(Clone)]
pub struct RpcService {
    chain_id: u64,
    bitcoin_network: u8,
    pub(crate) executor: ExecutorProxy,
    pub(crate) sequencer: SequencerProxy,
    pub(crate) indexer: IndexerProxy,
    pub(crate) pipeline_processor: PipelineProcessorProxy,
    pub(crate) bitcoin_client: Option<BitcoinClientProxy>,
    pub(crate) da_server: DAServerProxy,
}

impl RpcService {
    pub fn new(
        chain_id: u64,
        bitcoin_network: u8,
        executor: ExecutorProxy,
        sequencer: SequencerProxy,
        indexer: IndexerProxy,
        pipeline_processor: PipelineProcessorProxy,
        bitcoin_client: Option<BitcoinClientProxy>,
        da_server: DAServerProxy,
    ) -> Self {
        Self {
            chain_id,
            bitcoin_network,
            executor,
            sequencer,
            indexer,
            pipeline_processor,
            bitcoin_client,
            da_server,
        }
    }
}

impl RpcService {
    pub fn get_chain_id(&self) -> u64 {
        self.chain_id
    }

    pub fn get_bitcoin_network(&self) -> u8 {
        self.bitcoin_network
    }

    pub async fn queue_tx(&self, tx: RoochTransaction) -> Result<()> {
        //TODO implement queue tx and do not wait to execute
        let _ = self.execute_tx(tx).await?;
        Ok(())
    }

    pub async fn execute_tx(&self, tx: RoochTransaction) -> Result<ExecuteTransactionResponse> {
        self.pipeline_processor.execute_l2_tx(tx).await
    }

    pub async fn dry_run_tx(&self, tx: RoochTransactionData) -> Result<DryRunTransactionResult> {
        let verified_tx = self.executor.convert_to_verified_tx(tx).await?;
        self.executor.dry_run_transaction(verified_tx).await
    }

    pub async fn execute_view_function(
        &self,
        function_call: FunctionCall,
    ) -> Result<AnnotatedFunctionResult> {
        let module_id = function_call.function_id.module_id.clone();
        if !self.exists_module(module_id.clone()).await? {
            return Err(anyhow::anyhow!("Module does not exist: {}", module_id));
        }

        let resp = self.executor.execute_view_function(function_call).await?;
        Ok(resp)
    }

    pub async fn get_states(
        &self,
        access_path: AccessPath,
        state_root: Option<H256>,
    ) -> Result<Vec<Option<ObjectState>>> {
        self.executor.get_states(access_path, state_root).await
    }

    pub async fn exists_module(&self, module_id: ModuleId) -> Result<bool> {
        let mut resp = self
            .get_states(AccessPath::module(&module_id), None)
            .await?;
        Ok(resp.pop().flatten().is_some())
    }

    pub async fn get_annotated_states(
        &self,
        access_path: AccessPath,
    ) -> Result<Vec<Option<AnnotatedState>>> {
        self.executor.get_annotated_states(access_path).await
    }

    pub async fn list_states(
        &self,
        state_root: Option<H256>,
        access_path: AccessPath,
        cursor: Option<FieldKey>,
        limit: usize,
    ) -> Result<Vec<StateKV>> {
        self.executor
            .list_states(state_root, access_path, cursor, limit)
            .await
    }

    pub async fn list_annotated_states(
        &self,
        access_path: AccessPath,
        cursor: Option<FieldKey>,
        limit: usize,
    ) -> Result<Vec<AnnotatedStateKV>> {
        self.executor
            .list_annotated_states(access_path, cursor, limit)
            .await
    }

    pub async fn get_annotated_events_by_event_handle(
        &self,
        event_handle_type: StructTag,
        cursor: Option<u64>,
        limit: u64,
        descending_order: bool,
    ) -> Result<Vec<AnnotatedEvent>> {
        let resp = self
            .executor
            .get_annotated_events_by_event_handle(
                event_handle_type,
                cursor,
                limit,
                descending_order,
            )
            .await?;
        Ok(resp)
    }

    pub async fn get_events_by_event_handle(
        &self,
        event_handle_type: StructTag,
        cursor: Option<u64>,
        limit: u64,
        descending_order: bool,
    ) -> Result<Vec<Event>> {
        let resp = self
            .executor
            .get_events_by_event_handle(event_handle_type, cursor, limit, descending_order)
            .await?;
        Ok(resp)
    }

    pub async fn get_annotated_events_by_event_ids(
        &self,
        event_ids: Vec<EventID>,
    ) -> Result<Vec<Option<AnnotatedEvent>>> {
        let resp = self
            .executor
            .get_annotated_events_by_event_ids(event_ids)
            .await?;
        Ok(resp)
    }

    pub async fn get_events_by_event_ids(
        &self,
        event_ids: Vec<EventID>,
    ) -> Result<Vec<Option<Event>>> {
        let resp = self.executor.get_events_by_event_ids(event_ids).await?;
        Ok(resp)
    }

    pub async fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<LedgerTransaction>> {
        let resp = self.sequencer.get_transaction_by_hash(hash).await?;
        Ok(resp)
    }

    pub async fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<LedgerTransaction>>> {
        let resp = self.sequencer.get_transactions_by_hash(tx_hashes).await?;
        Ok(resp)
    }

    pub async fn get_tx_hashes(&self, tx_orders: Vec<u64>) -> Result<Vec<Option<H256>>> {
        let resp = self.sequencer.get_tx_hashes(tx_orders).await?;
        Ok(resp)
    }

    pub async fn get_transaction_execution_infos_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionExecutionInfo>>> {
        let resp = self
            .executor
            .get_transaction_execution_infos_by_hash(tx_hashes)
            .await?;
        Ok(resp)
    }

    pub async fn get_sequencer_order(&self) -> Result<u64> {
        let resp = self.sequencer.get_sequencer_order().await?;
        Ok(resp)
    }

    pub async fn query_transactions(
        &self,
        filter: TransactionFilter,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<u64>,
        limit: usize,
        descending_order: bool,
    ) -> Result<Vec<IndexerTransaction>> {
        let resp = self
            .indexer
            .query_transactions(filter, cursor, limit, descending_order)
            .await?;
        Ok(resp)
    }

    pub async fn query_events(
        &self,
        filter: EventFilter,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerEventID>,
        limit: usize,
        descending_order: bool,
    ) -> Result<Vec<IndexerEvent>> {
        let indexer_events = self
            .indexer
            .query_events(filter, cursor, limit, descending_order)
            .await?;

        let event_ids = indexer_events
            .iter()
            .map(|m| m.event_id.clone())
            .collect::<Vec<_>>();
        let events = self.get_events_by_event_ids(event_ids).await?;
        let result = indexer_events
            .into_iter()
            .zip(events)
            .filter_map(|(mut v, e_opt)| {
                match e_opt {
                    Some(e) => {
                        v.event_data = Some(e.event_data);
                        Some(v)
                    }
                    None => {
                        // Sometimes the indexer is delayed, maybe the event is deleted in the event store
                        tracing::trace!(
                            "Event {} in the indexer but can not found in event store",
                            v.event_id
                        );
                        None
                    }
                }
            })
            .collect::<Vec<_>>();

        Ok(result)
    }

    pub async fn query_annotated_events(
        &self,
        filter: EventFilter,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerEventID>,
        limit: usize,
        descending_order: bool,
    ) -> Result<Vec<AnnotatedIndexerEvent>> {
        let indexer_events = self
            .indexer
            .query_events(filter, cursor, limit, descending_order)
            .await?;

        let event_ids = indexer_events
            .iter()
            .map(|m| m.event_id.clone())
            .collect::<Vec<_>>();
        let events = self.get_annotated_events_by_event_ids(event_ids).await?;
        let result = indexer_events
            .into_iter()
            .zip(events)
            .filter_map(|(mut v, e_opt)| {
                match e_opt {
                    Some(e) => {
                        v.event_data = Some(e.event.event_data);
                        Some(AnnotatedIndexerEvent::new(v, e.decoded_event_data))
                    }
                    None => {
                        // Sometimes the indexer is delayed, maybe the event is deleted in the event store
                        tracing::trace!(
                            "Event {} in the indexer but can not found in event store",
                            v.event_id
                        );
                        None
                    }
                }
            })
            .collect::<Vec<_>>();

        Ok(result)
    }

    pub async fn query_object_states(
        &self,
        filter: ObjectStateFilter,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: usize,
        descending_order: bool,
        decode: bool,
        show_display: bool,
        state_type: ObjectStateType,
    ) -> Result<Vec<IndexerObjectStateView>> {
        let indexer_ids = match filter {
            // Compatible with object_ids query after split object_states
            // Do not query the indexer, directly return the states query results.
            ObjectStateFilter::ObjectId(object_ids) => {
                if object_ids.len() > MAX_OBJECT_IDS_PER_QUERY {
                    return Err(anyhow::anyhow!(
                        "Too many object IDs requested. Maximum allowed: {}",
                        MAX_OBJECT_IDS_PER_QUERY
                    ));
                }
                object_ids
                    .into_iter()
                    .map(|v| (v, IndexerStateID::default()))
                    .collect()
            }
            _ => {
                self.indexer
                    .query_object_ids(filter, cursor, limit, descending_order, state_type)
                    .await?
            }
        };
        let object_ids = indexer_ids.iter().map(|m| m.0.clone()).collect::<Vec<_>>();

        let access_path = AccessPath::objects(object_ids.clone());
        let mut object_states = if decode || show_display {
            let annotated_states = self.get_annotated_states(access_path).await?;
            let mut displays: BTreeMap<ObjectID, Option<DisplayFieldsView>> = if show_display {
                let valid_states = annotated_states
                    .iter()
                    .filter_map(|s| s.as_ref())
                    .collect::<Vec<&AnnotatedState>>();
                let valid_display_field_views = self
                    .get_display_fields_and_render(&valid_states, None)
                    .await?;
                valid_states
                    .iter()
                    .zip(valid_display_field_views)
                    .map(|(state, display_fields)| (state.metadata.id.clone(), display_fields))
                    .collect()
            } else {
                BTreeMap::new()
            };
            let mut object_states = annotated_states
                .into_iter()
                .zip(indexer_ids)
                .filter_map(|(state_opt, (object_id, indexer_state_id))| {
                    match state_opt {
                        Some(state) => Some(IndexerObjectStateView::try_new_from_annotated_state(
                            state,
                            indexer_state_id,
                        )),
                        None => {
                            // Sometimes the indexer is delayed, maybe the object is deleted in the state
                            tracing::trace!(
                                "Object {} in the indexer but can not found in state",
                                object_id
                            );
                            None
                        }
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            if !displays.is_empty() {
                object_states.iter_mut().for_each(|object_state| {
                    object_state.display_fields =
                        displays.remove(&object_state.metadata.id).flatten();
                });
            }
            object_states
        } else {
            let states = self.get_states(access_path, None).await?;
            states
                .into_iter()
                .zip(indexer_ids)
                .filter_map(|(state_opt, (object_id, indexer_state_id))| {
                    match state_opt {
                        Some(state) => Some(IndexerObjectStateView::new_from_object_state(
                            state,
                            indexer_state_id,
                        )),
                        None => {
                            // Sometimes the indexer is delayed, maybe the object is deleted in the state
                            tracing::trace!(
                                "Object {} in the indexer but can not found in state",
                                object_id
                            );
                            None
                        }
                    }
                })
                .collect::<Vec<_>>()
        };
        self.fill_bitcoin_addresses(object_states.iter_mut().map(|m| &mut m.metadata).collect())
            .await?;
        Ok(object_states)
    }

    pub async fn fill_bitcoin_addresses(
        &self,
        mut metadatas: Vec<&mut ObjectMetaView>,
    ) -> Result<()> {
        let bitcoin_network = self.bitcoin_network;
        let owners = metadatas.iter().map(|m| m.owner.0).collect::<Vec<_>>();
        let reverse_address_mapping = self.get_bitcoin_addresses(owners).await?;
        for metadata in metadatas.iter_mut() {
            let reverse_address = reverse_address_mapping
                .get(&metadata.owner.0)
                .cloned()
                .flatten();
            metadata.owner_bitcoin_address =
                reverse_address.and_then(|addr| addr.format(bitcoin_network).ok());
        }
        Ok(())
    }

    pub async fn get_bitcoin_addresses(
        &self,
        rooch_addresses: Vec<RoochAddress>,
    ) -> Result<HashMap<RoochAddress, Option<BitcoinAddress>>> {
        let mapping_object_id = RoochToBitcoinAddressMapping::object_id();
        let user_addresses = rooch_addresses
            .into_iter()
            .filter(|addr| !addr.is_vm_or_system_reserved_address())
            .collect::<Vec<_>>();
        let owner_keys = user_addresses
            .iter()
            .map(|addr| FieldKey::derive_from_address(&(*addr).into()))
            .collect::<Vec<_>>();

        let access_path = AccessPath::fields(mapping_object_id, owner_keys);
        let address_mapping = self
            .get_states(access_path, None)
            .await?
            .into_iter()
            .zip(user_addresses)
            .map(|(state_opt, owner)| {
                Ok((
                    owner,
                    state_opt
                        .map(|state| {
                            state
                                .value_as_df::<AccountAddress, BitcoinAddress>()
                                .map(|df| df.value)
                        })
                        .transpose()?,
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        Ok(address_mapping)
    }

    pub async fn get_display_fields_and_render(
        &self,
        states: &[&AnnotatedState],
        state_root: Option<H256>,
    ) -> Result<Vec<Option<DisplayFieldsView>>> {
        let mut display_ids = vec![];
        let mut displayable_states = vec![];
        for s in states {
            displayable_states.push(if !s.metadata.is_dynamic_field() {
                display_ids.push(get_object_display_id(s.metadata.object_type.clone()));
                true
            } else {
                //TODO should we support display for dynamic fields?
                false
            });
        }
        // get display fields
        let path = AccessPath::objects(display_ids);
        let mut display_fields = self
            .get_states(path, state_root)
            .await?
            .into_iter()
            .map(|option_s| {
                option_s
                    .map(|s| s.value_as_uncheck::<RawDisplay>())
                    .transpose()
            })
            .collect::<Result<Vec<Option<RawDisplay>>>>()?;
        display_fields.reverse();

        let mut display_field_views = vec![];
        for (annotated_s, displayable) in states.iter().zip(displayable_states) {
            display_field_views.push(if displayable {
                debug_assert!(
                    !display_fields.is_empty(),
                    "Display fields should not be empty"
                );
                display_fields.pop().unwrap().map(|display| {
                    DisplayFieldsView::new(display.render(
                        &annotated_s.metadata,
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

    pub async fn broadcast_bitcoin_transaction(
        &self,
        hex: String,
        maxfeerate: Option<f64>,
        maxburnamount: Option<f64>,
    ) -> Result<Txid> {
        let bitcoin_client = self
            .bitcoin_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Bitcoin client is not configured"))?;

        bitcoin_client
            .broadcast_transaction(hex, maxfeerate, maxburnamount)
            .await
    }

    pub async fn repair_indexer(
        &self,
        repair_type: RepairIndexerType,
        repair_params: RepairIndexerParams,
    ) -> Result<()> {
        {
            match repair_type {
                RepairIndexerType::ObjectState => match repair_params {
                    RepairIndexerParams::ObjectId(object_ids) => {
                        if object_ids.len() > MAX_OBJECT_IDS_PER_QUERY {
                            return Err(anyhow::anyhow!(
                                "Too many object IDs requested. Maximum allowed: {}",
                                MAX_OBJECT_IDS_PER_QUERY
                            ));
                        }
                        let states = self
                            .get_states(AccessPath::objects(object_ids.clone()), None)
                            .await?;
                        for state_type in [
                            ObjectStateType::ObjectState,
                            ObjectStateType::UTXO,
                            ObjectStateType::Inscription,
                        ] {
                            self.repair_indexer_object_states(
                                states.clone(),
                                &object_ids,
                                state_type,
                            )
                            .await?;
                        }

                        Ok(())
                    }
                    _ => Err(format_err!(
                        "Invalid params when repair indexer for ObjectState"
                    )),
                },
                RepairIndexerType::Transaction => {
                    Err(format_err!("Repair indexer for transaction not support"))
                }
                RepairIndexerType::Event => {
                    Err(format_err!("Repair indexer for event not support"))
                }
            }
        }
    }

    pub async fn repair_indexer_object_states(
        &self,
        states: Vec<Option<ObjectState>>,
        object_ids: &[ObjectID],
        state_type: ObjectStateType,
    ) -> Result<()> {
        {
            let mut remove_object_ids = vec![];
            let mut object_states_mapping = HashMap::new();
            for (idx, state_opt) in states.into_iter().enumerate() {
                match state_opt {
                    Some(state) => match state_type {
                        ObjectStateType::ObjectState => {
                            if !(type_tag_match(&state.metadata.object_type, &UTXO_TYPE_TAG)
                                && type_tag_match(
                                    &state.metadata.object_type,
                                    &INSCRIPTION_TYPE_TAG,
                                ))
                            {
                                object_states_mapping.insert(state.metadata.id.clone(), state);
                            }
                        }
                        ObjectStateType::UTXO => {
                            if type_tag_match(&state.metadata.object_type, &UTXO_TYPE_TAG) {
                                object_states_mapping.insert(state.metadata.id.clone(), state);
                            }
                        }
                        ObjectStateType::Inscription => {
                            if type_tag_match(&state.metadata.object_type, &INSCRIPTION_TYPE_TAG) {
                                object_states_mapping.insert(state.metadata.id.clone(), state);
                            }
                        }
                    },
                    None => remove_object_ids.push(object_ids[idx].clone()),
                }
            }

            let expect_update_object_ids: Vec<_> = object_states_mapping.keys().cloned().collect();
            let query_limit = expect_update_object_ids.len();
            let indexer_ids = self
                .indexer
                .query_object_ids(
                    ObjectStateFilter::ObjectId(expect_update_object_ids.clone()),
                    None,
                    query_limit,
                    true,
                    state_type.clone(),
                )
                .await?;

            let mut update_object_states = indexer_ids
                .into_iter()
                .map(|(object_id, indexer_state_id)| {
                    let state = object_states_mapping
                        .get(&object_id)
                        .ok_or(anyhow::anyhow!(
                            "Object states {:?} should exist",
                            object_id
                        ))?;
                    Ok(IndexerObjectState::new(
                        state.metadata.clone(),
                        indexer_state_id.tx_order,
                        indexer_state_id.state_index,
                    ))
                })
                .collect::<Result<Vec<_>>>()?;

            // Object state may exist in state, but not exist in indexer
            let actual_update_object_ids = update_object_states
                .iter()
                .map(|v| v.metadata.id.clone())
                .collect::<Vec<_>>();
            let new_object_ids = expect_update_object_ids
                .iter()
                .filter(|&v| !actual_update_object_ids.contains(v))
                .collect::<Vec<_>>();
            let mut new_object_states = if !new_object_ids.is_empty() {
                // set genesis tx_order and state_index_generator for new indexer repair
                let tx_order: u64 = 0;
                let last_state_index = self
                    .indexer
                    .query_last_state_index_by_tx_order(tx_order, state_type.clone())
                    .await?;
                let mut state_index_generator = last_state_index.map_or(0, |x| x + 1);
                new_object_ids
                    .into_iter()
                    .map(|k| {
                        let state = object_states_mapping
                            .get(k)
                            .ok_or(anyhow::anyhow!("Object states {:?} should exist", k))?;
                        let object_state = IndexerObjectState::new(
                            state.metadata.clone(),
                            tx_order,
                            state_index_generator,
                        );
                        state_index_generator += 1;
                        Ok(object_state)
                    })
                    .collect::<Result<Vec<_>>>()?
            } else {
                vec![]
            };

            update_object_states.append(&mut new_object_states);
            if !update_object_states.is_empty() {
                self.indexer
                    .persist_or_update_object_states(update_object_states, state_type.clone())
                    .await?;
            }

            if !remove_object_ids.is_empty() {
                self.indexer
                    .delete_object_states(remove_object_ids, state_type)
                    .await?
            }
            Ok(())
        }
    }

    pub async fn sync_states(
        &self,
        tx_orders: Vec<u64>,
        filter: SyncStateFilter,
    ) -> Result<Vec<StateChangeSetWithTxOrder>> {
        let states = self
            .executor
            .get_state_change_sets(tx_orders.clone())
            .await?
            .into_iter()
            .zip(tx_orders)
            .filter(|(x, _y)| x.is_some())
            .map(|(x, y)| StateChangeSetWithTxOrder::new(y, x.unwrap().state_change_set))
            .collect::<Vec<_>>();

        let result = match filter {
            SyncStateFilter::ObjectID(object_id) => states
                .into_iter()
                .filter_map(|s| {
                    let filter_changes = s
                        .state_change_set
                        .changes
                        .into_iter()
                        .filter(|(_, value)| value.metadata.id == object_id)
                        .collect();

                    let filter_state_change_set = StateChangeSet::new_with_changes(
                        s.state_change_set.state_root,
                        s.state_change_set.global_size,
                        filter_changes,
                    );

                    if !filter_state_change_set.changes.is_empty() {
                        Some(StateChangeSetWithTxOrder::new(
                            s.tx_order,
                            filter_state_change_set,
                        ))
                    } else {
                        None
                    }
                })
                .collect(),
            SyncStateFilter::All => states,
        };

        Ok(result)
    }

    pub async fn status(&self) -> Result<Status> {
        let service_status = self.pipeline_processor.get_service_status().await?;
        let sequencer_info = self.sequencer.get_sequencer_info().await?;
        let root_state = self.executor.get_root().await?;
        let da_server_status = self.da_server.get_status().await?;

        let rooch_status = RoochStatus {
            sequencer_info: sequencer_info.into(),
            root_state: root_state.into(),
            da_info: da_server_status.into(),
        };

        let pending_block = {
            let pending_block_module = self.executor.as_module_binding::<PendingBlockModule>();
            pending_block_module.get_best_block()?
        };
        let confirmed_block = {
            let bitcoin_module = self.executor.as_module_binding::<BitcoinModule>();
            bitcoin_module.get_latest_block()?
        };

        let bitcoin_status = BitcoinStatus {
            confirmed_block: confirmed_block.map(Into::into),
            pending_block: pending_block.map(Into::into),
        };

        Ok(Status {
            service_status,
            rooch_status,
            bitcoin_status,
        })
    }
}
