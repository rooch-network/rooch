// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::rpc_service::RpcService;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use moveos_types::access_path::AccessPath;
use moveos_types::h256::H256;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::moveos_std::raw_table::TableInfo;
use moveos_types::state::{AnnotatedKeyState, KeyState, PlaceholderStruct, State};
use moveos_types::state_resolver::{AnnotatedKeyStateKV, KeyStateKV};
use rooch_rpc_api::jsonrpc_types::account_view::BalanceInfoView;
use rooch_rpc_api::jsonrpc_types::CoinInfoView;
use rooch_types::address::{BitcoinAddress, MultiChainAddress};
use rooch_types::bitcoin::utxo::{UTXOState, UTXO};
use rooch_types::framework::account_coin_store::AccountCoinStoreModule;
use rooch_types::framework::address_mapping::AddressMapping;
use rooch_types::framework::coin::{CoinInfo, CoinModule};
use rooch_types::framework::coin_store::CoinStore;
use rooch_types::indexer::state::IndexerGlobalState;
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::transaction::{TransactionSequenceInfoMapping, TransactionWithInfo};
use std::collections::HashMap;

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
            .get_states(access_path)
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
                                    .as_object_uncheck::<CoinInfo<PlaceholderStruct>>()?
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
    ) -> Result<Vec<Option<CoinStore<PlaceholderStruct>>>> {
        let access_path = AccessPath::objects(coin_store_ids);
        self.rpc_service
            .get_states(access_path)
            .await?
            .into_iter()
            .map(|state_opt| {
                state_opt
                    .map(|state| {
                        Ok(state
                            .as_object_uncheck::<CoinStore<PlaceholderStruct>>()?
                            .value)
                    })
                    .transpose()
            })
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_balance(
        &self,
        account_addr: AccountAddress,
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

        let coin_store_id = AccountCoinStoreModule::account_coin_store_id(account_addr, coin_type);
        let balance = self
            .get_coin_stores(vec![coin_store_id])
            .await?
            .pop()
            .flatten()
            .map(|coin_store| coin_store.balance())
            .unwrap_or_default();

        Ok(BalanceInfoView::new(coin_info, balance))
    }

    pub async fn get_balances(
        &self,
        account_addr: AccountAddress,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<(Option<Vec<u8>>, BalanceInfoView)>> {
        let account_coin_store_module = self
            .rpc_service
            .executor
            .as_module_binding::<AccountCoinStoreModule>();
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
                        let coin_store_id = v.cast::<ObjectID>()?;
                        Ok((k, coin_store_id))
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
                    let balance_info =
                        BalanceInfoView::new(coin_info.clone(), coin_store.balance());
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

    pub async fn get_table_infos(
        &self,
        table_handles: Vec<ObjectID>,
    ) -> Result<HashMap<ObjectID, Option<TableInfo>>> {
        // Global table 0x0 table's key type is always ObjectID.
        let access_path = AccessPath::objects(table_handles.clone());
        self.rpc_service
            .get_states(access_path)
            .await?
            .into_iter()
            .zip(table_handles)
            .map(|(state_opt, table_handle)| {
                Ok((
                    table_handle,
                    state_opt
                        .map(|state| {
                            Ok::<TableInfo, anyhow::Error>(
                                state.as_object_uncheck::<TableInfo>()?.value,
                            )
                        })
                        .transpose()?,
                ))
            })
            .collect::<Result<HashMap<_, _>>>()
    }

    pub async fn list_states(
        &self,
        access_path: AccessPath,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<KeyStateKV>> {
        let states = self
            .rpc_service
            .list_states(access_path.clone(), cursor, limit)
            .await?;

        let (table_handle, _keys) = access_path.into_table_query();
        let table_infos = self.get_table_infos(vec![table_handle]).await?;
        // For now, global table 0x0 has no key type yet.
        let key_type_opt = table_infos
            .get(&table_handle)
            .cloned()
            .flatten()
            .map(|v| v.key_type_tag())
            .transpose()?;

        Ok(states
            .into_iter()
            .map(|(key, state)| {
                (
                    KeyState {
                        key,
                        key_type: key_type_opt.as_ref().cloned(),
                    },
                    state,
                )
            })
            .collect())
    }

    pub async fn list_annotated_states(
        &self,
        access_path: AccessPath,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<AnnotatedKeyStateKV>> {
        let states = self
            .rpc_service
            .list_annotated_states(access_path.clone(), cursor, limit)
            .await?;

        let (table_handle, _keys) = access_path.into_table_query();
        let table_infos = self.get_table_infos(vec![table_handle]).await?;

        let key_type_opt = table_infos
            .get(&table_handle)
            .cloned()
            .flatten()
            .map(|v| v.key_type_tag())
            .transpose()?;

        Ok(match key_type_opt {
            Some(key_type) => {
                let key_states = states
                    .iter()
                    .map(|(key, _state)| State::new(key.clone(), key_type.clone()))
                    .collect();
                let anotated_key_states = self
                    .rpc_service
                    .get_annotated_states_by_state(key_states)
                    .await?;
                states
                    .into_iter()
                    .zip(anotated_key_states)
                    .map(|((_key, state), key_state)| {
                        let key_state = AnnotatedKeyState::new(
                            KeyState::new(key_state.state.value, Some(key_state.state.value_type)),
                            Some(key_state.decoded_value),
                        );
                        (key_state, state)
                    })
                    .collect()
            }
            None => states
                .into_iter()
                .map(|(key, state)| {
                    let key_state = AnnotatedKeyState::new(KeyState::new(key, None), None);
                    (key_state, state)
                })
                .collect(),
        })
    }

    pub async fn pack_uxtos(&self, states: Vec<IndexerGlobalState>) -> Result<Vec<UTXOState>> {
        let table_handles = states
            .iter()
            .map(|m| m.object_id)
            .collect::<Vec<_>>();
        let owners = states
            .iter()
            .map(|m| m.owner)
            .collect::<Vec<_>>();
        let owner_keys = states
            .iter()
            .map(|m| m.owner.to_vec())
            .collect::<Vec<_>>();

        // Global table 0x0 table's key type is always ObjectID.
        let access_path = AccessPath::objects(table_handles.clone());
        let objects = self
            .rpc_service
            .get_states(access_path)
            .await?
            .into_iter()
            .zip(table_handles)
            .map(|(state_opt, table_handle)| {
                Ok((
                    table_handle,
                    state_opt
                        .map(|state| {
                            Ok::<UTXO, anyhow::Error>(state.as_object_uncheck::<UTXO>()?.value)
                        })
                        .transpose()?,
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let address_mapping_module = self
            .rpc_service
            .executor
            .as_module_binding::<AddressMapping>();
        let (_address_mapping_handle, _mapping_handle, reverse_mapping_handle) =
            address_mapping_module.address_mapping_handle()?;

        let access_path = AccessPath::table(reverse_mapping_handle, owner_keys);
        let reverse_address_mapping = self
            .rpc_service
            .get_states(access_path)
            .await?
            .into_iter()
            .zip(owners)
            .map(|(state_opt, owner)| {
                Ok((
                    owner,
                    state_opt
                        .map(|state| state.cast_unchecked::<Vec<MultiChainAddress>>())
                        .transpose()?,
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        debug_assert!(
            states.len() >= objects.len() && states.len() == reverse_address_mapping.len()
        );

        let data = states
            .into_iter()
            // .enumerate()
            .map(|state| {
                let utxo = objects
                    .get(&state.object_id)
                    .cloned()
                    .flatten()
                    .ok_or(anyhow::anyhow!("UTXO should have value"))?;
                let reverse_mapping_opt =
                    reverse_address_mapping.get(&state.owner).cloned().flatten();
                let reverse_address = reverse_mapping_opt.and_then(|m| {
                    m.iter()
                        .find(|v| v.multichain_id == RoochMultiChainID::Bitcoin)
                        .map(|p| BitcoinAddress::new(p.raw_address.clone()))
                });

                Ok(UTXOState::new_from_global_state(
                    state,
                    utxo,
                    reverse_address,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(data)
    }
}
