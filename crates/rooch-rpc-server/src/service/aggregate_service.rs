// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::rpc_service::RpcService;
use anyhow::Result;
use lazy_static::lazy_static;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use moveos_types::access_path::AccessPath;
use moveos_types::function_return_value::FunctionResult;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::move_types::get_first_ty_as_struct_tag;
use moveos_types::state_resolver::resource_tag_to_key;
use moveos_types::transaction::FunctionCall;
use moveos_types::tx_context::TxContext;
use rooch_rpc_api::api::MAX_RESULT_LIMIT_USIZE;
use rooch_types::account::BalanceInfo;
use rooch_types::addresses::ROOCH_FRAMEWORK_ADDRESS_LITERAL;
use rooch_types::framework::coin::{AnnotatedCoinInfo, AnnotatedCoinStore, CoinModule};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::runtime::Runtime;

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
    pub async fn get_balances(
        &self,
        account_addr: AccountAddress,
        coin_type: Option<StructTag>,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<Option<(Option<Vec<u8>>, BalanceInfo)>>> {
        let coin_module = self.as_module_binding::<CoinModule>();
        let coin_info_handle = coin_module.coin_info_handle()?;

        // contruct coin info map for handle decimals
        //TODO Extract to signle module as cache to load all token info, as well as to avoid query every time
        let mut coin_info_table = HashMap::new();
        let coin_info_states = self
            .rpc_service
            .list_annotated_states(
                AccessPath::table_without_keys(coin_info_handle),
                None,
                MAX_RESULT_LIMIT_USIZE,
            )
            .await?;
        let coin_info_data = coin_info_states
            .into_iter()
            .map(|item| {
                item.map(|(_key, state)| {
                    AnnotatedCoinInfo::new_from_annotated_move_value(state.move_value)
                })
                .transpose()
            })
            .collect::<Vec<_>>();

        for coin_info_opt in coin_info_data.into_iter().flatten() {
            if coin_info_opt.is_none() {
                return Err(anyhow::anyhow!("CoinInfo should have value"));
            };
            let coin_info = coin_info_opt.unwrap();

            let coin_type = get_first_ty_as_struct_tag(coin_info.get_type())
                .map_err(|e| anyhow::anyhow!("Coin type convert error :{}", e))?;

            coin_info_table.insert(coin_type, coin_info.value);
        }

        let mut result = vec![];

        let coin_store_handle_opt = coin_module.coin_store_handle(account_addr)?;
        if let Some(coin_store_handle) = coin_store_handle_opt {
            if let Some(coin_type) = coin_type {
                let coin_store_type = format!(
                    "{}::coin::CoinStore<0x{}>",
                    ROOCH_FRAMEWORK_ADDRESS_LITERAL,
                    coin_type.to_canonical_string()
                );
                let key = resource_tag_to_key(&StructTag::from_str(coin_store_type.as_str())?);
                let keys = vec![key];
                let mut states = self
                    .rpc_service
                    .get_annotated_states(AccessPath::table(coin_store_handle, keys))
                    .await?;

                let state_opt = states.pop().flatten();
                let state = state_opt
                    .ok_or_else(|| anyhow::anyhow!("CoinStore state should have value"))?;
                let annotated_coin_store =
                    AnnotatedCoinStore::new_from_annotated_move_value(state.move_value)?;

                let balance_info =
                    BalanceInfo::new_with_default(coin_type, annotated_coin_store.get_coin_value());
                result.push(Some((None, balance_info)))
            } else {
                //TODO If the coin store list exceeds MAX_RESULT_LIMIT_USIZE, consider supporting traverse or pagination
                let states = self
                    .rpc_service
                    .list_annotated_states(
                        AccessPath::table_without_keys(coin_store_handle),
                        cursor,
                        limit,
                    )
                    .await?;

                for (key, state) in states.into_iter().flatten() {
                    let coin_store =
                        AnnotatedCoinStore::new_from_annotated_move_value(state.move_value)
                            .map_err(|e| anyhow::anyhow!("CoinStore convert error :{}", e))?;

                    let coin_type = get_first_ty_as_struct_tag(coin_store.get_coin_type())
                        .map_err(|e| anyhow::anyhow!("Coin type convert should succ :{}", e))?;
                    let balance_info =
                        BalanceInfo::new_with_default(coin_type, coin_store.get_coin_value());
                    let v = (Some(key), balance_info);
                    result.push(Some(v))
                }
            };
        } else {
            // If the account do not exist, return None
            result.push(None)
        }

        for (_key, balance_info) in result.iter_mut().flatten() {
            let coin_info = coin_info_table
                .get(&balance_info.coin_type)
                .ok_or_else(|| anyhow::anyhow!("Get coin info by coin type should succ"))?;
            balance_info.symbol = coin_info.symbol.clone();
            balance_info.decimals = coin_info.decimals;
        }

        Ok(result)
    }
}

lazy_static! {
    static ref RUNTIME: Runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("rooch-aggregate-service")
        .enable_all()
        .build()
        .unwrap();
}

impl MoveFunctionCaller for AggregateService {
    // Use futures::executors::block_on to go from sync -> async
    // Warning! Possible deadlocks can occur if we try to wait for a future without spawn
    fn call_function(
        &self,
        _ctx: &TxContext,
        function_call: FunctionCall,
    ) -> Result<FunctionResult> {
        let rpc_service = self.rpc_service.clone();
        let function_result = futures::executor::block_on(
            RUNTIME.spawn(async move { rpc_service.execute_view_function(function_call).await }),
        )??;

        function_result.try_into()
    }
}
