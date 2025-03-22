// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::account_view::BalanceInfoView;
use crate::jsonrpc_types::address::UnitedAddressView;
use crate::jsonrpc_types::event_view::{EventFilterView, IndexerEventIDView, IndexerEventView};
use crate::jsonrpc_types::field_view::FieldFilterView;
use crate::jsonrpc_types::repair_view::{RepairIndexerParamsView, RepairIndexerTypeView};
use crate::jsonrpc_types::transaction_view::{TransactionFilterView, TransactionWithInfoView};
use crate::jsonrpc_types::{
    AccessPathView, AnnotatedFunctionResultView, BalanceInfoPageView, BytesView, EventOptions,
    EventPageView, ExecuteTransactionResponseView, FieldKeyView, FieldPageView, FunctionCallView,
    H256View, IndexerEventPageView, IndexerObjectStatePageView, IndexerStateIDView, ModuleABIView,
    ObjectIDVecView, ObjectIDView, ObjectStateFilterView, ObjectStateView, QueryOptions,
    RoochAddressView, StateChangeSetPageView, StateOptions, StatePageView, StrView, StructTagView,
    SyncStateFilterView, TransactionWithInfoPageView, TxOptions,
};
use crate::jsonrpc_types::{DryRunTransactionResponseView, Status};
use crate::RpcResult;
use jsonrpsee::core::SubscriptionResult;
use jsonrpsee::proc_macros::rpc;
use moveos_types::{access_path::AccessPath, state::FieldKey};
use rooch_open_rpc_macros::open_rpc;

#[open_rpc(namespace = "rooch")]
#[rpc(server, client, namespace = "rooch")]
#[async_trait]
pub trait RoochAPI {
    #[method(name = "getChainID")]
    async fn get_chain_id(&self) -> RpcResult<StrView<u64>>;

    /// Send the signed transaction in bcs hex format
    /// This method does not block waiting for the transaction to be executed.
    #[method(name = "sendRawTransaction")]
    async fn send_raw_transaction(&self, tx_bcs_hex: BytesView) -> RpcResult<H256View>;

    /// Send the signed transaction in bcs hex format
    /// This method blocks waiting for the transaction to be executed.
    #[method(name = "executeRawTransaction")]
    async fn execute_raw_transaction(
        &self,
        tx_bcs_hex: BytesView,
        tx_option: Option<TxOptions>,
    ) -> RpcResult<ExecuteTransactionResponseView>;

    #[method(name = "dryRunRawTransaction")]
    async fn dry_run(&self, tx_bcs_hex: BytesView) -> RpcResult<DryRunTransactionResponseView>;

    /// Execute a read-only function call
    /// The function do not change the state of Application
    #[method(name = "executeViewFunction")]
    async fn execute_view_function(
        &self,
        function_call: FunctionCallView,
    ) -> RpcResult<AnnotatedFunctionResultView>;

    /// Get the states by access_path
    /// If the StateOptions.decode is true, the state is decoded and the decoded value is returned in the response.
    #[method(name = "getStates")]
    async fn get_states(
        &self,
        access_path: AccessPathView,
        state_option: Option<StateOptions>,
    ) -> RpcResult<Vec<Option<ObjectStateView>>>;

    /// List the states by access_path
    /// If the StateOptions.decode is true, the state is decoded and the decoded value is returned in the response.
    #[method(name = "listStates")]
    async fn list_states(
        &self,
        access_path: AccessPathView,
        cursor: Option<String>,
        limit: Option<StrView<u64>>,
        state_option: Option<StateOptions>,
    ) -> RpcResult<StatePageView>;

    /// Get object states by object id
    #[method(name = "getObjectStates")]
    async fn get_object_states(
        &self,
        object_ids: ObjectIDVecView,
        state_option: Option<StateOptions>,
    ) -> RpcResult<Vec<Option<ObjectStateView>>>;

    /// Get Object Fields via ObjectID and field keys.
    #[method(name = "getFieldStates")]
    async fn get_field_states(
        &self,
        object_id: ObjectIDView,
        field_key: Vec<FieldKeyView>,
        state_option: Option<StateOptions>,
    ) -> RpcResult<Vec<Option<ObjectStateView>>> {
        let key_states = field_key.into_iter().map(FieldKey::from).collect();
        let access_path_view =
            AccessPathView::from(AccessPath::fields(object_id.into(), key_states));
        self.get_states(access_path_view, state_option).await
    }

    /// List Object Fields via ObjectID.
    #[method(name = "listFieldStates")]
    async fn list_field_states(
        &self,
        object_id: ObjectIDView,
        cursor: Option<String>,
        limit: Option<StrView<u64>>,
        state_option: Option<StateOptions>,
    ) -> RpcResult<StatePageView> {
        let access_path_view =
            AccessPathView::from(AccessPath::fields_without_keys(object_id.into()));
        self.list_states(access_path_view, cursor, limit, state_option)
            .await
    }

    /// Get the events by event handle id
    #[method(name = "getEventsByEventHandle")]
    async fn get_events_by_event_handle(
        &self,
        event_handle_type: StructTagView,
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
        descending_order: Option<bool>,
        event_options: Option<EventOptions>,
    ) -> RpcResult<EventPageView>;

    #[method(name = "getTransactionsByHash")]
    async fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256View>,
    ) -> RpcResult<Vec<Option<TransactionWithInfoView>>>;

    #[method(name = "getTransactionsByOrder")]
    async fn get_transactions_by_order(
        &self,
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
        descending_order: Option<bool>,
    ) -> RpcResult<TransactionWithInfoPageView>;

    /// get account balance by RoochAddress and CoinType
    #[method(name = "getBalance")]
    async fn get_balance(
        &self,
        account_addr: UnitedAddressView,
        coin_type: StructTagView,
    ) -> RpcResult<BalanceInfoView>;

    /// get account balances by RoochAddress
    #[method(name = "getBalances")]
    async fn get_balances(
        &self,
        account_addr: UnitedAddressView,
        cursor: Option<IndexerStateIDView>,
        limit: Option<StrView<u64>>,
    ) -> RpcResult<BalanceInfoPageView>;

    /// get module ABI by module id
    #[method(name = "getModuleABI")]
    async fn get_module_abi(
        &self,
        module_addr: RoochAddressView,
        module_name: String,
    ) -> RpcResult<Option<ModuleABIView>>;

    /// Query the transactions indexer by transaction filter
    #[method(name = "queryTransactions")]
    async fn query_transactions(
        &self,
        filter: TransactionFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
        query_option: Option<QueryOptions>,
    ) -> RpcResult<TransactionWithInfoPageView>;

    /// Query the events indexer by event filter
    #[method(name = "queryEvents")]
    async fn query_events(
        &self,
        filter: EventFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerEventIDView>,
        limit: Option<StrView<u64>>,
        query_option: Option<QueryOptions>,
    ) -> RpcResult<IndexerEventPageView>;

    /// Query the object states indexer by state filter
    #[method(name = "queryObjectStates")]
    async fn query_object_states(
        &self,
        filter: ObjectStateFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateIDView>,
        limit: Option<StrView<u64>>,
        query_option: Option<QueryOptions>,
    ) -> RpcResult<IndexerObjectStatePageView>;

    /// Query the fields indexer by field filter
    #[method(name = "queryFields")]
    async fn query_fields(
        &self,
        filter: FieldFilterView,
        page: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
        query_option: Option<QueryOptions>,
    ) -> RpcResult<FieldPageView>;

    /// Repair indexer by sync from states
    #[method(name = "repairIndexer")]
    async fn repair_indexer(
        &self,
        repair_type: RepairIndexerTypeView,
        repair_params: RepairIndexerParamsView,
    ) -> RpcResult<()>;

    /// Sync state change sets
    #[method(name = "syncStates")]
    async fn sync_states(
        &self,
        filter: SyncStateFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
        query_option: Option<QueryOptions>,
    ) -> RpcResult<StateChangeSetPageView>;

    /// Get the chain and service status
    #[method(name = "status")]
    async fn status(&self) -> RpcResult<Status>;

    /// Check change sets from sync states
    #[method(name = "checkChangeSets")]
    async fn check_change_set(
        &self,
        cursor: Option<StrView<u64>>,
        limit: Option<StrView<u64>>,
        query_option: Option<QueryOptions>,
    ) -> RpcResult<Vec<u64>>;

    /// Subscribe to a stream of event
    #[subscription(name = "subscribeEvents", item = IndexerEventView)]
    fn subscribe_events(&self, filter: EventFilterView) -> SubscriptionResult;

    /// Subscribe to a stream of transaction with execution info
    #[subscription(name = "subscribeTransactions", item = TransactionWithInfoView)]
    fn subscribe_transactions(&self, filter: TransactionFilterView) -> SubscriptionResult;
}
