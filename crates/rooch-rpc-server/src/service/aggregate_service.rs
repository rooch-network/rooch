// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::rpc_service::RpcService;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use moveos_types::access_path::AccessPath;
use moveos_types::function_return_value::FunctionResult;
use moveos_types::h256::H256;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::moveos_std::event::EventModule;
use moveos_types::moveos_std::object::{Object, ObjectID};
use moveos_types::moveos_std::tx_context::TxContext;
use moveos_types::state_resolver::resource_tag_to_key;
use moveos_types::transaction::FunctionCall;
use rooch_types::account::BalanceInfo;
use rooch_types::framework::account_coin_store::AccountCoinStoreModule;
use rooch_types::framework::coin::{CoinInfo, CoinModule};
use rooch_types::framework::coin_store::CoinStore;
use rooch_types::transaction::{TransactionSequenceInfoMapping, TransactionWithInfo};
use std::collections::HashMap;
use tokio::runtime::Handle;

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
    ) -> Result<HashMap<StructTag, Option<CoinInfo>>> {
        let coin_module = self.as_module_binding::<CoinModule>();
        let coin_info_handle = coin_module.coin_infos_handle()?;

        let access_path = AccessPath::table(
            coin_info_handle,
            coin_types.iter().map(resource_tag_to_key).collect(),
        );
        self.rpc_service
            .get_states(access_path)
            .await?
            .into_iter()
            .zip(coin_types)
            .map(|(state_opt, coin_type)| {
                Ok((
                    coin_type,
                    state_opt
                        .map(|state| state.as_move_state::<CoinInfo>())
                        .transpose()?,
                ))
            })
            .collect::<Result<HashMap<_, _>>>()
    }

    pub async fn get_coin_stores(
        &self,
        coin_store_ids: Vec<ObjectID>,
    ) -> Result<Vec<Option<CoinStore>>> {
        let access_path = AccessPath::objects(coin_store_ids);
        self.rpc_service
            .get_states(access_path)
            .await?
            .into_iter()
            .map(|state_opt| {
                state_opt
                    .map(|state| Ok(state.as_object::<CoinStore>()?.value))
                    .transpose()
            })
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_balance(
        &self,
        account_addr: AccountAddress,
        coin_type: StructTag,
    ) -> Result<BalanceInfo> {
        let account_coin_store_module = self.as_module_binding::<AccountCoinStoreModule>();

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
            account_coin_store_module.coin_store_id(account_addr, coin_type.clone())?;
        let balance = match coin_store_id {
            Some(coin_store_id) => {
                let coin_store = self
                    .get_coin_stores(vec![coin_store_id])
                    .await?
                    .pop()
                    .flatten()
                    .ok_or_else(|| {
                        anyhow::anyhow!("Can not find CoinStore with id: {}", coin_store_id)
                    })?;
                coin_store.balance()
            }
            None => move_core_types::u256::U256::zero(),
        };
        Ok(BalanceInfo::new(coin_info, balance))
    }

    pub async fn get_balances(
        &self,
        account_addr: AccountAddress,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<(Option<Vec<u8>>, BalanceInfo)>> {
        let account_coin_store_module = self.as_module_binding::<AccountCoinStoreModule>();
        let coin_stores_handle_opt = account_coin_store_module.coin_stores_handle(account_addr)?;

        match coin_stores_handle_opt {
            Some(coin_stores_handle) => {
                let coin_store_ids = self
                    .rpc_service
                    .list_states(
                        AccessPath::table_without_keys(coin_stores_handle),
                        cursor,
                        limit,
                    )
                    .await?
                    .into_iter()
                    .map(|(k, v)| {
                        let coin_store_ref = v.as_move_state::<Object<CoinStore>>()?;
                        Ok((k, coin_store_ref.id))
                    })
                    .collect::<Result<Vec<_>>>()?;

                let coin_stores = self
                    .get_coin_stores(coin_store_ids.iter().map(|(_, v)| *v).collect())
                    .await?;

                let coin_types = coin_stores
                    .iter()
                    .flatten()
                    .map(|coin_store| coin_store.coin_type_tag())
                    .collect::<Vec<_>>();

                let coin_info_map = self.get_coin_infos(coin_types).await?;

                let mut result = vec![];
                for ((key, object_id), coin_store) in coin_store_ids.into_iter().zip(coin_stores) {
                    let coin_store = coin_store.ok_or_else(|| {
                        anyhow::anyhow!("Can not find CoinStore with id: {}", object_id)
                    })?;
                    let coin_info = coin_info_map
                        .get(&coin_store.coin_type_tag())
                        .cloned()
                        .flatten()
                        .ok_or_else(|| {
                            anyhow::anyhow!("Can not find CoinInfo for {}", coin_store.coin_type())
                        })?;
                    let balance_info = BalanceInfo::new(coin_info.clone(), coin_store.balance());
                    result.push((Some(key), balance_info))
                }

                Ok(result)
            }
            None => Ok(vec![]),
        }
    }

    pub async fn get_transaction_with_info(
        &self,
        tx_hashes: Vec<H256>,
        tx_sequence_info_mapping: Vec<Option<TransactionSequenceInfoMapping>>,
    ) -> Result<Vec<Option<TransactionWithInfo>>> {
        // If the tx hash is invalid, filled None when returned.
        let tx_orders = tx_sequence_info_mapping
            .clone()
            .iter()
            .flatten()
            .map(|m| m.tx_order)
            .collect();

        let transactions = self
            .rpc_service
            .get_transactions_by_hash(tx_hashes.clone())
            .await?;

        let sequence_infos = self
            .rpc_service
            .get_transaction_sequence_infos(tx_orders)
            .await?;

        let execution_infos = self
            .rpc_service
            .get_transaction_execution_infos_by_hash(tx_hashes.clone())
            .await?;

        debug_assert!(
            transactions.len() >= sequence_infos.len()
                && transactions.len() == execution_infos.len()
        );
        let sequence_info_map = sequence_infos
            .into_iter()
            .flatten()
            .map(|sequence_info| (sequence_info.tx_order, sequence_info))
            .collect::<HashMap<_, _>>(); // collect into a hashmap

        tx_sequence_info_mapping
            .iter()
            .enumerate()
            .map(|(index, tx_mapping_opt)| {
                match tx_mapping_opt {
                    Some(tx_mapping) => {
                        let sequence_info = match sequence_info_map.get(&tx_mapping.tx_order) {
                            Some(v) => v.clone(),
                            None => {
                                return Err(anyhow::anyhow!(
                                    "TransactionSequenceInfo should exist when construct TransactionWithInfo"
                                ))
                            }
                        };
                        Ok(Some(TransactionWithInfo {
                            transaction: transactions[index].clone().ok_or(anyhow::anyhow!(
                                "Transaction should exist when construct TransactionWithInfo"
                            ))?,
                            sequence_info,
                            execution_info: execution_infos[index].clone().ok_or(anyhow::anyhow!(
                                "TransactionExecutionInfo should exist when construct TransactionWithInfo"
                            ))?,
                        }))
                    },
                    None => Ok(None),
                }
            })
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_event_handle(
        &self,
        event_handle_type: StructTag,
    ) -> Result<(ObjectID, AccountAddress, u64)> {
        let event_module = self.as_module_binding::<EventModule>();
        event_module.get_event_handle(event_handle_type)
    }
}

impl MoveFunctionCaller for AggregateService {
    fn call_function(
        &self,
        _ctx: &TxContext,
        function_call: FunctionCall,
    ) -> Result<FunctionResult> {
        let rpc_service = self.rpc_service.clone();
        let function_result = tokio::task::block_in_place(|| {
            Handle::current()
                .block_on(async move { rpc_service.execute_view_function(function_call).await })
        })?;
        function_result.try_into()
    }
}
