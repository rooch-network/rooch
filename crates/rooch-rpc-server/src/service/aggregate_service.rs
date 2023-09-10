// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::rpc_service::RpcService;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use moveos_types::access_path::AccessPath;
use moveos_types::function_return_value::FunctionResult;
use moveos_types::module_binding::{ModuleBinding, MoveFunctionCaller};
use moveos_types::move_option::MoveOption;
use moveos_types::move_types::get_first_ty_as_struct_tag;
use moveos_types::object::ObjectID;
use moveos_types::state::MoveState;
use moveos_types::state_resolver::resource_tag_to_key;
use moveos_types::transaction::FunctionCall;
use moveos_types::tx_context::TxContext;
use rooch_rpc_api::api::MAX_RESULT_LIMIT_USIZE;
use rooch_types::account::BalanceInfo;
use rooch_types::addresses::ROOCH_FRAMEWORK_ADDRESS_LITERAL;
use rooch_types::framework::coin::{AnnotatedCoinInfo, AnnotatedCoinStore, CoinModule};
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
    pub async fn get_balances(
        &self,
        account_addr: AccountAddress,
        coin_type: Option<StructTag>,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<Option<(Option<Vec<u8>>, BalanceInfo)>>> {
        // let call = FunctionCall::new(
        //     CoinModule::function_id(CoinModule::COIN_STORE_HANDLE_FUNCTION_NAME),
        //     vec![],
        //     vec![account_addr.to_vec()],
        // );
        // let coin_store_handle_opt: Option<ObjectID> = client
        //     .call_function(&ctx, call)?
        //     .into_result()
        //     .map(|values| {
        //         let value = values.get(0).expect("Expected return value");
        //         let result = MoveOption::<ObjectID>::from_bytes(&value.value)
        //             .expect("Expected Option<ObjectID>");
        //         result
        //     })?;
        // let coin_module = self.as_module_binding::<CoinModule>();

        // contruct coin info map for handle decimals
        // let coin_info_handle =
        //     tokio::task::spawn_blocking(move || coin_module.coin_info_handle().ok()?)
        //         .await
        //         .map_err(|e| RoochError::AccountBalanceError(e.to_string()))?;
        // let self_service = coin_module.coin_info_handle().clone();
        // let coin_info_handle = tokio::task::spawn_blocking(
        //     coin_module.coin_info_handle()?
        // );
        // let coin_info_handle = coin_info_handle.expect("Get coin info handle should succ");

        // // contruct coin info map for handle decimals
        // let coin_info_handle = coin_module
        //     .coin_info_handle()?
        //     .expect("Get coin info handle should succ");

        let coin_info_handle_call = FunctionCall::new(
            CoinModule::function_id(CoinModule::COIN_INFO_HANDLE_FUNCTION_NAME),
            vec![],
            vec![],
        );
        let ctx = TxContext::new_readonly_ctx(account_addr);
        let coin_info_handle: Option<ObjectID> = self
            .call_function(&ctx, coin_info_handle_call)?
            .into_result()
            .map(|values| {
                let value = values
                    .get(0)
                    .expect("Coin info handle expected return value");
                let result = MoveOption::<ObjectID>::from_bytes(&value.value)
                    .expect("Coin info handle Expected Option<ObjectID>");
                result.into()
            })?;
        let coin_info_handle = coin_info_handle.expect("Get coin info handle should succ");

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
                        .expect("AnnotatedCoinInfo expected return value")
                })
            })
            .collect::<Vec<_>>();
        for coin_info in coin_info_data.into_iter().flatten() {
            coin_info_table.insert(coin_info.struct_type, coin_info.value);
        }

        let coin_store_handle_call = FunctionCall::new(
            CoinModule::function_id(CoinModule::COIN_STORE_HANDLE_FUNCTION_NAME),
            vec![],
            vec![account_addr.to_vec()],
        );
        // let ctx = TxContext::new_readonly_ctx(account_addr);
        let coin_store_handle: Option<ObjectID> = self
            .call_function(&ctx, coin_store_handle_call)?
            .into_result()
            .map(|values| {
                let value = values
                    .get(0)
                    .expect("Coin store handle expected return value");
                let result = MoveOption::<ObjectID>::from_bytes(&value.value)
                    .expect("Coin store handle expected Option<ObjectID>");
                result.into()
            })?;
        let coin_store_handle = coin_store_handle.expect("Get coin store handle should succ");
        // let coin_store_handle = coin_module
        //     .coin_store_handle(account_addr)?
        //     .expect("Get coin store handle should succ");

        // let mut account_info = AccountInfo::new(0u64, vec![]);
        let mut result = vec![];
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

            let state = states.pop().flatten().expect("State expected return value");
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

            let mut data = states
                .into_iter()
                .map(|item| {
                    item.map(|(key, state)| {
                        let coin_store =
                            AnnotatedCoinStore::new_from_annotated_move_value(state.move_value)
                                .expect("AnnotatedCoinStore expected return value");

                        let coin_type =
                            get_first_ty_as_struct_tag(coin_store.get_coin_struct_type())
                                .expect("Coin type expected get_first_ty_as_struct_tag succ");
                        // let inner_coin_info = coin_info_table
                        //     .get(&coin_type)
                        //     .expect("Get coin info by coin type expected return value");
                        let balance_info =
                            BalanceInfo::new_with_default(coin_type, coin_store.get_coin_value());
                        (Some(key), balance_info)
                    })
                })
                .collect::<Vec<_>>();

            result.append(&mut data);
        };

        for (_key, balance_info) in result.iter_mut().flatten() {
            let coin_info = coin_info_table
                .get(&balance_info.coin_type)
                .expect("Get coin info by coin type expected return value");
            balance_info.symbol = coin_info.symbol.clone();
            balance_info.decimals = coin_info.decimals;
        }

        Ok(result)
    }
}

impl MoveFunctionCaller for AggregateService {
    // Use futures::executors::block_on to go from sync -> async
    // Warning! Possible deadlocks can occur if we try to wait for a future
    fn call_function(
        &self,
        _ctx: &TxContext,
        function_call: FunctionCall,
    ) -> Result<FunctionResult> {
        // let function_result = futures::executor::block_on(async move {
        // let function_result = futures::executor::block_on(async move {
        //     self.rpc_service.execute_view_function(function_call).await
        // })?;

        let function_result =
            futures::executor::block_on(self.rpc_service.execute_view_function(function_call))?;

        // let rt = tokio::runtime::Builder::new_current_thread()
        //     .enable_all()
        //     .build()?;
        // let function_result = rt.block_on(self.rpc_service.execute_view_function(function_call))?;

        // let function_result = tokio::task::spawn_blocking(move || self.rpc_service.execute_view_function(function_call));

        // let handle = Handle::current();

        // let function_result = handle.block_on(self.rpc_service.execute_view_function(function_call))?;
        // let handle = tokio::runtime::Handle::current();
        // let function_result = std::thread::scope(|s| {
        //     s.spawn(|| {
        //         handle.block_on(self.rpc_service.execute_view_function(function_call))
        //     })
        //         .join()
        // })?.unwrap();
        // .unwrap();

        // let function_result = Handle::block_on ( async move {
        //         self.rpc_service.execute_view_function(function_call).await
        // });
        // let function_result = block_in_place ( async move {
        //         self.rpc_service.execute_view_function(function_call).await
        // })?;

        // let handle = tokio::runtime::Handle::current();
        // let rpc_service = self.rpc_service.clone().;
        // let function_result = futures::executor::block_on(async {
        //     handle
        //         .spawn(async { rpc_service.execute_view_function(function_call).await })
        //         .await
        //         .expect("Task spawned in Tokio executor panicked")
        // })?;

        // Warning! Possible deadlocks can occur if we try to wait for a future
        // let rpc_service = self.rpc_service.clone();
        // let function_result = futures::executor::block_on(
        //     tokio::task::spawn(
        //         rpc_service.execute_view_function(function_call)
        //     )
        //     // .unwrap()
        // )??;

        // // Warning! Possible deadlocks can occur if we try to wait for a future
        // let function_result = futures::executor::block_on(async move {
        //     // tokio::task::spawn(
        //     RUNTIME.spawn(
        //         async move { rpc_service.execute_view_function(function_call).await },
        //     )
        //     .await?
        // })?;

        // let task = tokio::task::spawn(self.rpc_service.execute_view_function(function_call));
        // let function_result = futures::executor::block_on(tokio::task::spawn(
        //     self.rpc_service.execute_view_function(function_call)
        // ))?;

        // let function_result =
        //     futures::executor::block_on( self.rpc_service.execute_view_function(function_call))?;

        // let function_result =
        //     tokio::task::block_in_place(|| {
        //         self.rpc_service.execute_view_function(function_call)
        //     }).;

        // let result = tokio::task::spawn_blocking(invoke_juniper).await.map_err(|e| e.to_string())?;
        // println!("result={:?}", result);
        // Ok(json!({}))

        // let result = tokio::task::spawn_blocking(async || {
        //     self.rpc_service.execute_view_function(function_call).await
        // }).unwrap()?;

        function_result.try_into()
    }
}
