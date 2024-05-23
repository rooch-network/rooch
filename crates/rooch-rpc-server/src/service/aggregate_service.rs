// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::rpc_service::RpcService;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::{StructTag, TypeTag};
use moveos_types::access_path::AccessPath;
use moveos_types::h256::H256;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::moveos_std::object::RawObject;
use moveos_types::state::{KeyState, PlaceholderStruct};
use rooch_rpc_api::jsonrpc_types::account_view::BalanceInfoView;
use rooch_rpc_api::jsonrpc_types::CoinInfoView;
use rooch_types::address::{BitcoinAddress, MultiChainAddress};
use rooch_types::bitcoin::ord::{Inscription, InscriptionState};
use rooch_types::bitcoin::utxo::{UTXOState, UTXO};
use rooch_types::framework::account_coin_store::AccountCoinStoreModule;
use rooch_types::framework::address_mapping::AddressMappingModule;
use rooch_types::framework::coin::{CoinInfo, CoinModule};
use rooch_types::framework::coin_store::CoinStore;
use rooch_types::indexer::state::IndexerObjectState;
use rooch_types::multichain_id::RoochMultiChainID;
use rooch_types::transaction::TransactionWithInfo;
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
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<(Option<KeyState>, BalanceInfoView)>> {
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
                        AccessPath::fields_without_keys(coin_stores_handle),
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
                    .get_coin_stores(coin_store_ids.iter().map(|(_, v)| v.clone()).collect())
                    .await?;

                let coin_types = coin_stores
                    .iter()
                    .flatten()
                    .map(|coin_store| coin_store.coin_type())
                    .collect::<Vec<_>>();

                let coin_info_map = self.get_coin_infos(coin_types).await?;

                let mut result = vec![];
                for ((key, object_id), coin_store) in coin_store_ids.into_iter().zip(coin_stores) {
                    let coin_store = coin_store.ok_or_else(|| {
                        anyhow::anyhow!("Can not find CoinStore with id: {}", object_id)
                    })?;
                    let coin_info = coin_info_map
                        .get(&coin_store.coin_type())
                        .cloned()
                        .flatten()
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Can not find CoinInfo for {}",
                                coin_store.coin_type_str()
                            )
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
                    execution_info: exec_info,
                })),
                _ => Ok(None),
            })
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_raw_objects(
        &self,
        object_ids: Vec<ObjectID>,
    ) -> Result<HashMap<ObjectID, Option<RawObject>>> {
        // Global table 0x0 table's key type is always ObjectID.
        let access_path = AccessPath::objects(object_ids.clone());
        self.rpc_service
            .get_states(access_path)
            .await?
            .into_iter()
            .zip(object_ids)
            .map(|(state_opt, object_id)| {
                Ok((
                    object_id,
                    match state_opt {
                        Some(state) => Some(state.as_raw_object()?),
                        None => None,
                    },
                ))
            })
            .collect::<Result<HashMap<_, _>>>()
    }

    pub async fn pack_uxtos(&self, states: Vec<IndexerObjectState>) -> Result<Vec<UTXOState>> {
        let object_ids = states
            .iter()
            .map(|m| m.object_id.clone())
            .collect::<Vec<_>>();
        let owners = states.iter().map(|m| m.owner).collect::<Vec<_>>();
        let owner_keys = states
            .iter()
            .map(|m| KeyState::new(m.owner.to_vec(), TypeTag::Address))
            .collect::<Vec<_>>();

        // Global table 0x0 table's key type is always ObjectID.
        let access_path = AccessPath::objects(object_ids.clone());
        let objects = self
            .rpc_service
            .get_states(access_path)
            .await?
            .into_iter()
            .zip(object_ids)
            .map(|(state_opt, object_id)| {
                Ok((
                    object_id,
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
            .as_module_binding::<AddressMappingModule>();
        let (_address_mapping_handle, _mapping_handle, reverse_mapping_handle) =
            address_mapping_module.address_mapping_handle()?;

        let access_path = AccessPath::fields(reverse_mapping_handle, owner_keys);
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

        let data = states
            .into_iter()
            .map(|state| {
                let utxo = objects.get(&state.object_id).cloned().flatten();
                let reverse_mapping_opt =
                    reverse_address_mapping.get(&state.owner).cloned().flatten();
                let reverse_address = reverse_mapping_opt.and_then(|m| {
                    m.iter()
                        .find(|v| v.multichain_id == RoochMultiChainID::Bitcoin)
                        .map(|p| BitcoinAddress::new(p.raw_address.clone()))
                });

                Ok(UTXOState::new_from_object_state(
                    state,
                    utxo,
                    reverse_address,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(data)
    }

    pub async fn pack_inscriptions(
        &self,
        states: Vec<IndexerObjectState>,
    ) -> Result<Vec<InscriptionState>> {
        let object_ids = states
            .iter()
            .map(|m| m.object_id.clone())
            .collect::<Vec<_>>();
        let owners = states.iter().map(|m| m.owner).collect::<Vec<_>>();
        let owner_keys = states
            .iter()
            .map(|m| KeyState::new(m.owner.to_vec(), TypeTag::Address))
            .collect::<Vec<_>>();

        // Global table 0x0 table's key type is always ObjectID.
        let access_path = AccessPath::objects(object_ids.clone());
        let objects = self
            .rpc_service
            .get_states(access_path)
            .await?
            .into_iter()
            .zip(object_ids)
            .map(|(state_opt, object_id)| {
                Ok((
                    object_id,
                    state_opt
                        .map(|state| {
                            Ok::<Inscription, anyhow::Error>(
                                state.as_object_uncheck::<Inscription>()?.value,
                            )
                        })
                        .transpose()?,
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let address_mapping_module = self
            .rpc_service
            .executor
            .as_module_binding::<AddressMappingModule>();
        let (_address_mapping_handle, _mapping_handle, reverse_mapping_handle) =
            address_mapping_module.address_mapping_handle()?;

        let access_path = AccessPath::fields(reverse_mapping_handle, owner_keys);
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

        let data = states
            .into_iter()
            // .enumerate()
            .map(|state| {
                let inscription = objects
                    .get(&state.object_id)
                    .cloned()
                    .flatten()
                    .ok_or(anyhow::anyhow!("Inscription should have value"))?;
                let reverse_mapping_opt =
                    reverse_address_mapping.get(&state.owner).cloned().flatten();
                let reverse_address = reverse_mapping_opt.and_then(|m| {
                    m.iter()
                        .find(|v| v.multichain_id == RoochMultiChainID::Bitcoin)
                        .map(|p| BitcoinAddress::new(p.raw_address.clone()))
                });

                Ok(InscriptionState::new_from_object_state(
                    state,
                    inscription,
                    reverse_address,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(data)
    }
}
