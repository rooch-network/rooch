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
use rooch_rpc_api::jsonrpc_types::{AccountAddressView, AnnotatedCoinStoreView, StructTagView};
use rooch_types::account::BalanceInfo;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::framework::coin::CoinModule;

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
impl CommandAction<()> for BalanceCommand {
    async fn execute(self) -> RoochResult<()> {
        let context = self.context_options.build().await?;

        let addr_view: Option<AccountAddress> = if let Some(address) = self.address {
            Some(address.into())
        } else {
            context.config.active_address.map(|address| address.into())
        };

        // Obtain account address
        let addr = addr_view.expect("Account not found error");

        let client = context.get_client().await?;

        let ctx = TxContext::new_readonly_ctx(addr);
        let call = FunctionCall::new(
            CoinModule::function_id(CoinModule::COIN_STORE_HANDLE_FUNCTION_NAME),
            vec![],
            vec![addr.to_vec()],
        );
        let coin_store_handle_opt: Option<ObjectID> = client
            .call_function(&ctx, call)?
            .into_result()
            .map(|values| {
                let value = values.get(0).expect("Expected return value");
                let result = MoveOption::<ObjectID>::from_bytes(&value.value)
                    .expect("Expected Option<ObjectID>");
                result.into()
            })?;
        let coin_store_handle = coin_store_handle_opt
            // .expect(format!("Failed to get coin store handle via {}", addr).as_str());
            .unwrap_or_else(|| panic!("Failed to get coin store handle via {}", addr));

        let mut result = AccountInfoView::new(0u64, vec![]);
        if let Some(coin_type_opt) = self.coin_type {
            let key = resource_tag_to_key(&coin_type_opt.0);
            let _hex_key = hex::encode(key.clone());
            let keys = vec![key];
            let mut states = client
                .get_annotated_states(AccessPath::table(coin_store_handle, keys))
                .await
                .map_err(|e| RoochError::AccountBalanceError(e.to_string()))?;

            let state = states
                .pop()
                .expect("States expected return value")
                .expect("State expected return value");

            let annotated_coin_store_view =
                AnnotatedCoinStoreView::new_from_annotated_move_value_view(state.move_value)?;

            let balance_info = BalanceInfo::new(
                annotated_coin_store_view.get_coin_type().into(),
                annotated_coin_store_view.get_coin_value().into(),
            );
            result.balances.push(Some(balance_info.into()))
        } else {
            let states = client
                .list_annotated_states(
                    AccessPath::table_without_keys(coin_store_handle).into(),
                    None,
                    Some(MAX_RESULT_LIMIT_USIZE),
                )
                .await
                .map_err(|e| RoochError::AccountBalanceError(e.to_string()))?;

            let mut annotated_coin_store_views = states
                .data
                .into_iter()
                .map(|state| {
                    let coin_store_view =
                        AnnotatedCoinStoreView::new_from_annotated_move_value_view(
                            state.expect("State expected return value").move_value,
                        )
                        .expect("AnnotatedCoinStoreView expected return value");

                    let balance_info = BalanceInfo::new(
                        coin_store_view.get_coin_type().into(),
                        coin_store_view.get_coin_value().into(),
                    );
                    Some(balance_info.into())
                })
                .collect();

            result.balances.append(&mut annotated_coin_store_views);
        };

        println!("{0: ^102} | {1: ^32}", "Coin Type", "Balance");
        println!("{}", ["-"; 48].join(""));
        for balance_info in result.balances.into_iter().flatten() {
            println!(
                "{0: ^102} | {1: ^32}",
                balance_info.coin_type, balance_info.balance,
            );
        }

        Ok(())
    }
}
