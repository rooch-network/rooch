// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use rooch_rpc_api::api::MAX_RESULT_LIMIT_USIZE;
use rooch_rpc_api::jsonrpc_types::{AccountAddressView, StructTagView};
use rooch_types::error::RoochResult;

/// Show account balance, only the accounts managed by the current node are supported
#[derive(Debug, Parser)]
pub struct BalanceCommand {
    #[clap(short = 'a', long = "address")]
    /// The account's address to show balance, if absent, show the default active account.
    address: Option<AccountAddressView>,

    /// Struct name as `<ADDRESS>::<MODULE_ID>::<STRUCT_NAME><TypeParam>`
    /// Example: `0x3::gas_coin::GasCoin`, `0x123::Coin::Box<0x123::coin_box::FCoin>`
    #[clap(long)]
    coin_type: Option<StructTagView>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<()> for BalanceCommand {
    async fn execute(self) -> RoochResult<()> {
        let context = self.context_options.build().await?;
        let address_addr = self
            .address
            .map_or(
                context
                    .client_config
                    .active_address
                    .map(|active_address| AccountAddress::from(active_address).into()),
                Some,
            )
            .expect("Account not found error");

        let client = context.get_client().await?;

        let data = client
            .get_balances(
                address_addr,
                self.coin_type,
                None,
                Some(MAX_RESULT_LIMIT_USIZE),
            )
            .await?;

        println!(
            "{0: ^102} | {1: ^16} | {2: ^32} | {3: ^6}",
            "Coin Type", "Symbol", "Balance", "Decimals"
        );
        println!("{}", ["-"; 68].join(""));

        for balance_info in data.data.into_iter().flatten() {
            println!(
                "{0: ^102} | {1: ^16} | {2: ^32} | {3: ^6}",
                balance_info.coin_type,
                balance_info.symbol,
                balance_info.balance,
                balance_info.decimals
            );
        }

        Ok(())
    }
}
