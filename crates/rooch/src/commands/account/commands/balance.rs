// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use moveos_types::access_path::AccessPath;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::state_resolver::resource_tag_to_key;
use rooch_rpc_api::api::MAX_RESULT_LIMIT_USIZE;
use rooch_rpc_api::jsonrpc_types::account_view::AccountInfoView;
use rooch_rpc_api::jsonrpc_types::{AccountAddressView, StructTagView};
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
    async fn execute(self) -> RoochResult<AccountInfoView> {
        let mut context = self.context_options.build().await?;

        let addr_view = if let Some(address) = self.address {
            self.address
        } else {
            context.config.active_address.map(|address| address.into().into())
        };

        // Obtain account address
        let addr = AccountAddress::from(addr_view.expect("Account not found error"));

        let client = context.get_client().await?;
        let coin_module = client.as_module_binding::<CoinModule>();
        let coin_store_handle = coin_module.coin_store_handle(addr)?;

        let mut result = AccountInfoView::new(0u64, vec![]);
        if coin_store_handle.is_some() {
            let _resp = if let Some(coin_type) = self.coin_type {
                client
                    .list_annotated_states(
                        AccessPath::table_without_keys(coin_store_handle.unwrap()).into(),
                        None,
                        Some(MAX_RESULT_LIMIT_USIZE),
                    )
                    .await
                    .map_err(|e| RoochError::AccountBalanceError(e.to_string()))?;
                // .pop()
                // .flatten();
                result.balances.push(Some(BalanceInfo::random().into()))
            } else {
                let keys = vec![resource_tag_to_key(&self.coin_type.unwrap().into())];
                let _state = client
                    .get_annotated_states(
                        AccessPath::table(coin_store_handle.unwrap(), keys).into(),
                    )
                    .await
                    .map_err(|e| RoochError::AccountBalanceError(e.to_string()))?;
                // result.balances.append(BalanceInfo {
                //     coin_type: TypeInfo::,
                //     balance,
                // }
                result.balances.push(Some(BalanceInfo::random().into()))
            };
        }
        Ok(result)
    }
}
