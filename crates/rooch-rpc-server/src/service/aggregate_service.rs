// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::rpc_service::RpcService;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use moveos_types::access_path::AccessPath;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::FieldKey;
use moveos_types::state::PlaceholderStruct;
use rooch_config::settings::is_multi_coin_store_enabled;
use rooch_rpc_api::jsonrpc_types::account_view::BalanceInfoView;
use rooch_rpc_api::jsonrpc_types::CoinInfoView;
use rooch_types::address::RoochAddress;
use rooch_types::framework::account_coin_store::AccountCoinStoreModule;
use rooch_types::framework::coin::{CoinInfo, CoinModule};
use rooch_types::framework::coin_store::{CoinStore, CoinStoreInfo};
use rooch_types::framework::multi_coin_store::CoinStoreField;
use rooch_types::indexer::state::{IndexerStateID, ObjectStateFilter, ObjectStateType};
use rooch_types::indexer::transaction::IndexerTransaction;
use rooch_types::transaction::TransactionWithInfo;
use std::collections::HashMap;
use std::str::FromStr;

/// AggregateService is aggregate RPC service and MoveFunctionCaller.
#[derive(Clone)]
pub struct AggregateService {
    rpc_service: RpcService,
}

impl AggregateService {
    pub fn new(rpc_service: RpcService) -> Self {
        Self { rpc_service }
    }
}

impl AggregateService {
    pub async fn get_coin_infos(
        &self,
        coin_types: Vec<StructTag>,
    ) -> Result<HashMap<StructTag, Option<CoinInfoView>>> {
        let access_path = AccessPath::objects(
            coin_types
                .iter()
                .cloned()
                .map(CoinModule::coin_info_id)
                .collect(),
        );
        self.rpc_service
            .get_states(access_path, None)
            .await?
            .into_iter()
            .zip(coin_types)
            .map(|(state_opt, coin_type)| {
                Ok((
                    coin_type,
                    state_opt
                        .map(|state| {
                            Ok::<CoinInfoView, anyhow::Error>(CoinInfoView::from(
                                state
                                    .into_object_uncheck::<CoinInfo<PlaceholderStruct>>()?
                                    .value,
                            ))
                        })
                        .transpose()?,
                ))
            })
            .collect::<Result<HashMap<_, _>>>()
    }

    pub async fn get_coin_stores(
        &self,
        coin_store_ids: Vec<ObjectID>,
    ) -> Result<Vec<Option<CoinStoreInfo>>> {
        let access_path = AccessPath::objects(coin_store_ids);
        self.rpc_service
            .get_states(access_path, None)
            .await?
            .into_iter()
            .map(|state_opt| state_opt.map(CoinStoreInfo::try_from).transpose())
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_multi_coin_store_fields(
        &self,
        field_with_coin_store_ids: Vec<(ObjectID, StructTag)>,
    ) -> Result<Vec<Option<CoinStoreInfo>>> {
        // Group fields by object_id to minimize RPC calls
        let mut object_map: HashMap<ObjectID, Vec<(FieldKey, usize)>> = HashMap::new();

        // Map each field to its original index in the input vector
        for (idx, (object_id, field)) in field_with_coin_store_ids.iter().enumerate() {
            let field_key = FieldKey::derive_from_string(field.to_canonical_string().as_str());
            object_map
                .entry(object_id.clone())
                .or_default()
                .push((field_key, idx));
        }

        // Create a results vector with the same size as the input
        let mut results = Vec::with_capacity(field_with_coin_store_ids.len());
        // Initialize with None values
        for _ in 0..field_with_coin_store_ids.len() {
            results.push(None);
        }

        // Process each object_id and its fields
        for (object_id, fields_with_indices) in object_map {
            // Extract just the field keys for the RPC call
            let field_keys: Vec<FieldKey> =
                fields_with_indices.iter().map(|(key, _)| *key).collect();

            // Construct access path for this object's fields
            let access_path = AccessPath::fields(object_id, field_keys);

            // Fetch states for all fields in a single RPC call
            let state_kvs = self.rpc_service.get_states(access_path, None).await?;

            // Map states to CoinStoreInfo and place them in the correct positions
            for ((_, original_idx), state_opt) in fields_with_indices.into_iter().zip(state_kvs) {
                let coin_store_info = state_opt
                    .map(|state| {
                        let coin_store_field = CoinStoreField::try_from(state)?;
                        CoinStoreInfo::try_from(coin_store_field)
                    })
                    .transpose()?;

                results[original_idx] = coin_store_info;
            }
        }

        Ok(results)
    }

    pub async fn get_balance(
        &self,
        account_addr: RoochAddress,
        coin_type: StructTag,
    ) -> Result<BalanceInfoView> {
        let coin_info = self
            .get_coin_infos(vec![coin_type.clone()])
            .await?
            .into_values()
            .flatten()
            .next()
            .ok_or_else(|| {
                anyhow::anyhow!("Can not find CoinInfo with coin_type: {}", coin_type)
            })?;

        let coin_store_id =
            AccountCoinStoreModule::account_coin_store_id(account_addr.into(), coin_type.clone());
        let mut balance = self
            .get_coin_stores(vec![coin_store_id])
            .await?
            .pop()
            .flatten()
            .map(|coin_store| coin_store.balance())
            .unwrap_or_default();

        // Compatible logic, query coin store and multi coin store at the same time
        if is_multi_coin_store_enabled() {
            let multi_coin_store_balance = self
                .get_balance_by_type_name(account_addr, coin_type)
                .await?;
            balance += multi_coin_store_balance.balance.0;
        }

        Ok(BalanceInfoView::new(coin_info, balance))
    }

    pub async fn get_balance_by_type_name(
        &self,
        account_addr: RoochAddress,
        coin_type: StructTag,
    ) -> Result<BalanceInfoView> {
        let coin_info = self
            .get_coin_infos(vec![coin_type.clone()])
            .await?
            .into_values()
            .flatten()
            .next()
            .ok_or_else(|| {
                anyhow::anyhow!("Can not find CoinInfo with coin_type: {}", coin_type)
            })?;

        let coin_store_id = AccountCoinStoreModule::multi_coin_store_id(account_addr.into());
        let balance = self
            .get_multi_coin_store_fields(vec![(coin_store_id, coin_type.clone())])
            .await?
            .pop()
            .flatten()
            .map(|coin_store| coin_store.balance())
            .unwrap_or_default();

        Ok(BalanceInfoView::new(coin_info, balance))
    }

    pub async fn query_account_coin_stores(
        &self,
        owner: AccountAddress,
        cursor: Option<IndexerStateID>,
        limit: usize,
    ) -> Result<Vec<(ObjectID, IndexerStateID)>> {
        self.rpc_service
            .indexer
            .query_object_ids(
                ObjectStateFilter::ObjectTypeWithOwner {
                    object_type: CoinStore::struct_tag_without_coin_type(),
                    filter_out: false,
                    owner,
                },
                cursor,
                limit,
                false,
                ObjectStateType::ObjectState,
            )
            .await
    }

    pub async fn get_balances(
        &self,
        owner: AccountAddress,
        cursor: Option<IndexerStateID>,
        limit: usize,
    ) -> Result<Vec<(Option<IndexerStateID>, BalanceInfoView)>> {
        let indexer_coin_stores = self.query_account_coin_stores(owner, cursor, limit).await?;
        let coin_store_ids = indexer_coin_stores
            .iter()
            .map(|(id, _state_id)| id.clone())
            .collect::<Vec<_>>();

        let coin_stores = self.get_coin_stores(coin_store_ids.clone()).await?;

        let coin_types = coin_stores
            .iter()
            .flatten()
            .map(|coin_store| coin_store.coin_type())
            .collect::<Vec<_>>();

        let coin_info_map = self.get_coin_infos(coin_types.clone()).await?;

        // Pre-fetch multi-coin store balances if the feature is enabled
        let multi_coin_store_balances: HashMap<StructTag, CoinStoreInfo> =
            if is_multi_coin_store_enabled() {
                let multi_coin_store_id = AccountCoinStoreModule::multi_coin_store_id(owner);

                // Create field_with_coin_store_ids for all coin types
                let field_with_coin_store_ids: Vec<(ObjectID, StructTag)> = coin_types
                    .iter()
                    .map(|coin_type| (multi_coin_store_id.clone(), coin_type.clone()))
                    .collect();

                // Batch query all fields at once
                if !field_with_coin_store_ids.is_empty() {
                    let multi_coin_store_fields = self
                        .get_multi_coin_store_fields(field_with_coin_store_ids)
                        .await?;

                    // Build a map of coin_type -> balance
                    coin_types
                        .into_iter()
                        .zip(multi_coin_store_fields)
                        .filter_map(|(coin_type, coin_store_info_opt)| {
                            coin_store_info_opt.map(|info| (coin_type, info))
                        })
                        .collect()
                } else {
                    HashMap::new()
                }
            } else {
                HashMap::new()
            };

        let mut result = vec![];
        for ((id, state_id), coin_store) in indexer_coin_stores.into_iter().zip(coin_stores) {
            let coin_store = coin_store
                .ok_or_else(|| anyhow::anyhow!("Can not find CoinStore with id: {}", id))?;
            let coin_info = coin_info_map
                .get(&coin_store.coin_type())
                .cloned()
                .flatten()
                .ok_or_else(|| {
                    anyhow::anyhow!("Can not find CoinInfo for {}", coin_store.coin_type_str())
                })?;

            let mut balance = coin_store.balance();

            // Add multi-coin store balance from cache if available
            if is_multi_coin_store_enabled() {
                if let Some(coin_store_info) =
                    multi_coin_store_balances.get(&coin_store.coin_type())
                {
                    balance += coin_store_info.balance();
                }
            }

            let balance_info = BalanceInfoView::new(coin_info.clone(), balance);
            result.push((Some(state_id), balance_info))
        }

        Ok(result)
    }

    pub async fn get_balances_by_type_name(
        &self,
        owner: AccountAddress,
        cursor: Option<FieldKey>,
        limit: usize,
    ) -> Result<Vec<(Option<FieldKey>, BalanceInfoView)>> {
        // Get the multi-coin store ID for the owner
        let multi_coin_store_id = AccountCoinStoreModule::multi_coin_store_id(owner);

        // Construct access path for fields of the multi-coin store
        let access_path = AccessPath::object(multi_coin_store_id);
        // List all fields (or fields after cursor) with pagination
        let field_states = self
            .rpc_service
            .list_states(None, access_path, cursor, limit)
            .await?;

        // Process each field state
        let mut results = Vec::new();
        let mut coin_types = Vec::new();
        let mut coin_store_infos = Vec::new();
        let mut field_keys = Vec::new();

        for (field_key, state) in field_states {
            // Convert field state to CoinStoreField and then to CoinStoreInfo
            match CoinStoreField::try_from(state) {
                Ok(coin_store_field) => {
                    match CoinStoreInfo::try_from(coin_store_field) {
                        Ok(coin_store_info) => {
                            // Extract the coin type from the coin store info
                            let struct_tag = StructTag::from_str(&coin_store_info.coin_type_str())
                                .map_err(|_| anyhow::anyhow!("Invalid coin type string"))?;

                            coin_types.push(struct_tag);
                            coin_store_infos.push(coin_store_info);
                            field_keys.push(field_key);
                        }
                        Err(e) => {
                            // Skip invalid conversion but log error
                            tracing::error!("Error converting to CoinStoreInfo: {}", e);
                        }
                    }
                }
                Err(e) => {
                    // Skip invalid conversion but log error
                    tracing::error!("Error converting to CoinStoreField: {}", e);
                }
            }
        }

        // Get coin info for all coin types in a single batch
        let coin_info_map = self.get_coin_infos(coin_types.clone()).await?;

        // Build the final result
        for ((field_key, coin_store_info), coin_type) in
            field_keys.into_iter().zip(coin_store_infos).zip(coin_types)
        {
            let coin_info = coin_info_map
                .get(&coin_type)
                .cloned()
                .flatten()
                .ok_or_else(|| anyhow::anyhow!("Can not find CoinInfo for {}", coin_type))?;

            let balance_info = BalanceInfoView::new(coin_info, coin_store_info.balance());
            results.push((Some(field_key), balance_info));
        }

        Ok(results)
    }

    pub async fn get_transaction_with_info(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionWithInfo>>> {
        let transactions = self
            .rpc_service
            .get_transactions_by_hash(tx_hashes.clone())
            .await?;

        let execution_infos = self
            .rpc_service
            .get_transaction_execution_infos_by_hash(tx_hashes.clone())
            .await?;

        debug_assert!(transactions.len() == execution_infos.len());

        transactions
            .into_iter()
            .zip(execution_infos)
            .map(|(tx_opt, exec_info_opt)| match (tx_opt, exec_info_opt) {
                (Some(tx), Some(exec_info)) => Ok(Some(TransactionWithInfo {
                    transaction: tx,
                    execution_info: Some(exec_info),
                })),
                (Some(tx), None) => Ok(Some(TransactionWithInfo {
                    transaction: tx,
                    execution_info: None,
                })),
                _ => Ok(None),
            })
            .collect::<Result<Vec<_>>>()
    }

    pub async fn build_transaction_with_infos(
        &self,
        indexer_txs: Vec<IndexerTransaction>,
    ) -> Result<Vec<TransactionWithInfo>> {
        let tx_hashes = indexer_txs.iter().map(|m| m.tx_hash).collect::<Vec<_>>();
        let ledger_txs = self
            .rpc_service
            .get_transactions_by_hash(tx_hashes.clone())
            .await?;
        let execution_infos = self
            .rpc_service
            .get_transaction_execution_infos_by_hash(tx_hashes)
            .await?;

        let data = indexer_txs
            .into_iter()
            .zip(ledger_txs)
            .zip(execution_infos)
            .map(|((_indexer_tx, ledger_tx_opt), execution_info_opt)| {
                let ledger_tx =
                    ledger_tx_opt.ok_or(anyhow::anyhow!("LedgerTransaction should have value"))?;
                let execution_info = execution_info_opt.ok_or(anyhow::anyhow!(
                    "TransactionExecutionInfo should have value"
                ))?;
                Ok(TransactionWithInfo::new(ledger_tx, execution_info))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(data)
    }
}
