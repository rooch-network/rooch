// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use move_command_line_common::address::ParsedAddress;
use move_command_line_common::types::ParsedStructType;
use rooch_rpc_api::api::MAX_RESULT_LIMIT_USIZE;
use rooch_types::error::RoochResult;

/// Show account balance, only the accounts managed by the current node are supported
#[derive(Debug, Parser)]
pub struct BalanceCommand {
    #[clap(short = 'a', long = "address", parse(try_from_str = ParsedAddress::parse), default_value = "default")]
    /// The account's address to show balance, if absent, show the default active account.
    address: ParsedAddress,

    /// Struct name as `<ADDRESS>::<MODULE_ID>::<STRUCT_NAME><TypeParam>`
    /// Example: `0x3::gas_coin::GasCoin`, `0x123::Coin::Box<0x123::coin_box::FCoin>`
    #[clap(long, parse(try_from_str = ParsedStructType::parse))]
    coin_type: Option<ParsedStructType>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<()> for BalanceCommand {
    async fn execute(self) -> RoochResult<()> {
        let context = self.context_options.build()?;
        let mapping = context.address_mapping();
        let address_addr = self.address.into_account_address(&mapping)?;
        let coin_type = self
            .coin_type
            .map(|t| t.into_struct_tag(&mapping))
            .transpose()?;
        let client = context.get_client().await?;

        let balances = match coin_type {
            Some(coin_type) => {
                vec![
                    client
                        .rooch
                        .get_balance(address_addr.into(), coin_type.into())
                        .await?,
                ]
            }
            None => {
                client
                    .rooch
                    .get_balances(address_addr.into(), None, Some(MAX_RESULT_LIMIT_USIZE))
                    .await?
                    .data
            }
        };

        println!(
            "{0: ^102} | {1: ^16} | {2: ^6} |  {3: ^32} ",
            "Coin Type", "Symbol", "Decimals", "Balance"
        );
        println!("{}", ["-"; 68].join(""));

        for balance_info in balances {
            println!(
                "{0: ^102} | {1: ^16} | {2: ^6} | {3: ^32} ",
                balance_info.coin_info.coin_type,
                balance_info.coin_info.symbol,
                balance_info.coin_info.decimals,
                balance_info.balance,
            );
        }

        Ok(())
    }
}
