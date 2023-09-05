// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use moveos_types::access_path::AccessPath;
use moveos_types::module_binding::{ModuleBinding, MoveFunctionCaller};
use moveos_types::move_option::MoveOption;
use moveos_types::object::ObjectID;
use moveos_types::state::MoveStructState;
use moveos_types::state_resolver::resource_tag_to_key;
use moveos_types::transaction::FunctionCall;
use moveos_types::tx_context::TxContext;
use rooch_rpc_api::api::MAX_RESULT_LIMIT_USIZE;
use rooch_rpc_api::jsonrpc_types::account_view::AccountInfoView;
use rooch_rpc_api::jsonrpc_types::{AccountAddressView, StructTagView};
use rooch_types::account::BalanceInfo;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::framework::coin::{AnnotatedCoinStore, CoinModule};

/// Show a account info, only the accounts managed by the current node are supported
#[derive(Debug, Parser)]
pub struct BalanceCommand {
    #[clap(short = 'a', long = "address")]
    /// The account's address to show balance, if absent, show the default active account.
    address: Option<AccountAddressView>,

    /// Struct name as `<ADDRESS>::<MODULE_ID>::<STRUCT_NAME><TypeParam1?, TypeParam2?>`
    /// Example: `0x123::counter::Counter`, `0x123::counter::Box<0x123::counter::Counter>`
    #[clap(long = "coin_type")]
    /// The block number or block hash for get state, if absent, use latest block state_root.
    coin_type: Option<StructTagView>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<AccountInfoView> for BalanceCommand {
    async fn execute(self) -> RoochResult<AccountInfoView> {
        let context = self.context_options.build().await?;

        let addr_view: Option<AccountAddress> = if let Some(address) = self.address {
            Some(address.into())
        } else {
            context.config.active_address.map(|address| address.into())
        };

        // Obtain account address
        let addr = addr_view.expect("Account not found error");

        let client = context.get_client().await?;
        // let coin_module = client.as_module_binding::<CoinModule>();
        // let coin_store_handle = coin_module.coin_store_handle(addr)?;

        let ctx = TxContext::new_readonly_ctx(addr);
        let call = FunctionCall::new(
            CoinModule::function_id(CoinModule::COIN_STORE_HANDLE_FUNCTION_NAME),
            vec![],
            vec![addr.to_vec()],
        );
        let coin_store_handle: Option<ObjectID> = client
            .call_function(&ctx, call)?
            .into_result()
            .map(|values| {
                let value = values.get(0).expect("Expected return value");
                let result = MoveOption::<ObjectID>::from_bytes(&value.value)
                    .expect("Expected Option<ObjectID>");
                result.into()
                // result
            })
            .expect(format!("Failed to get coin store handle via {}", addr).as_str());
        // .unwrap_or_else(|_| {
        //     RoochError::ViewFunctionError(format!(
        //         "Failed to get coin store handle via {}",
        //         addr
        //     ))
        // });
        println!("account balance coin_store_handle {:?}", coin_store_handle);

        // let call = FunctionCall::new(
        //     Self::function_id(Self::GET_AUTHENTICATION_KEY_FUNCTION_NAME),
        //     vec![V::type_tag()],
        //     vec![MoveValue::Address(address)
        //         .simple_serialize()
        //         .expect("address should serialize")],
        // );
        // let ctx = TxContext::new_readonly_ctx(address);
        // let auth_key =client
        //     .call_function(&ctx, call)?
        //     .into_result()
        //     .map(|values| {

        let mut result = AccountInfoView::new(0u64, vec![]);
        if coin_store_handle.is_some() {
            let _resp = if let Some(_coin_type) = self.coin_type {
                client
                    .list_annotated_states(
                        AccessPath::table_without_keys(coin_store_handle.clone().unwrap()).into(),
                        None,
                        Some(MAX_RESULT_LIMIT_USIZE),
                    )
                    .await
                    .map_err(|e| RoochError::AccountBalanceError(e.to_string()))?;
                // .pop()
                // .flatten();

                // "jsonrpc": "2.0",
                // "result": {
                //     "data": [
                //     {
                //         "state": {
                //         "value": "0x102700000000000000000000000000000000000000000000000000000000000000",
                //         "value_type": "0x3::coin::CoinStore<0xe1176537c0175d336353dad12f7eb60c658ce526eeb3cd08409e6fd8c2dfa1d7::fixed_supply_coin::FSC>"
                //     },
                //         "move_value": {
                //         "abilities": 8,
                //         "type": "0x3::coin::CoinStore<0xe1176537c0175d336353dad12f7eb60c658ce526eeb3cd08409e6fd8c2dfa1d7::fixed_supply_coin::FSC>",
                //         "value": {
                //             "coin": {
                //                 "abilities": 4,
                //                 "type": "0x3::coin::Coin<0xe1176537c0175d336353dad12f7eb60c658ce526eeb3cd08409e6fd8c2dfa1d7::fixed_supply_coin::FSC>",
                //                 "value": {
                //                     "value": "10000"
                //                 }
                //             },
                //             "frozen": false
                //         }
                //     }
                //     },

                result.balances.push(Some(BalanceInfo::random().into()))
            } else {
                let keys = vec![resource_tag_to_key(&self.coin_type.unwrap().into())];
                let states = client
                    .get_annotated_states(
                        AccessPath::table(coin_store_handle.unwrap(), keys).into(),
                    )
                    .await
                    .map_err(|e| RoochError::AccountBalanceError(e.to_string()))?;
                // result.balances.append(BalanceInfo {
                //     coin_type: TypeInfo::,
                //     balance,
                // }

                let state = states
                    .get(0)
                    .expect("States expected return value")
                    .expect("State expected return value");
                // let result = MoveOption::<ObjectID>::from_bytes(&value.value)
                //     .expect("Expected Option<ObjectID>");

                let annotated_coin_store =
                    AnnotatedCoinStore::new_from_annotated_struct(state.into())?;

                let balance_info = BalanceInfo::new(
                    annotated_coin_store.get_coin_type(),
                    annotated_coin_store.get_coin_value(),
                );
                println!("Debug Account balance info {:?}", balance_info);
                result.balances.push(Some(balance_info.into()))
            };
        }
        Ok(result)
    }
}
