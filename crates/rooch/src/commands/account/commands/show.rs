// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use moveos_types::access_path::AccessPath;
use moveos_types::moveos_std::account::Account;
use moveos_types::state::ObjectState;
use rooch_rpc_api::jsonrpc_types::{AccountAddressView, BitcoinAddressView, StrView};
use rooch_types::{address::ParsedAddress, error::RoochResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tabled::settings::peaker::PriorityRight;
use tabled::settings::{Height, Width};
use tabled::{
    builder::Builder,
    settings::{Panel, Style},
};
use terminal_size::{terminal_size, Height as TerminalHeight, Width as TerminalWidth};

/// Show account info (account address and sequence number) on Rooch Network and bitcoin address on Bitcoin. Requires internet connection and works without rooch init.
#[derive(Debug, Parser)]
pub struct ShowCommand {
    #[clap(short = 'a', long = "address", value_parser=ParsedAddress::parse, default_value = "default")]
    address: ParsedAddress,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct ShowResultView {
    pub account_address: Option<AccountAddressView>,
    pub sequence_number: Option<StrView<u64>>,
    pub bitcoin_address: Option<BitcoinAddressView>,
}

impl ShowResultView {
    pub fn new(
        account_address: Option<AccountAddressView>,
        sequence_number: Option<StrView<u64>>,
        bitcoin_address: Option<BitcoinAddressView>,
    ) -> Self {
        // show result view with account (account address and sequence number) and bitcoin address
        ShowResultView {
            account_address,
            sequence_number,
            bitcoin_address,
        }
    }
}

#[async_trait]
impl CommandAction<Option<ShowResultView>> for ShowCommand {
    async fn execute(self) -> RoochResult<Option<ShowResultView>> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;
        let mapping = context.address_mapping();
        let rooch_address = self.address.clone().into_rooch_address(&mapping)?;
        let account_address = self.address.clone().into_account_address(&mapping)?;
        let account_opt = client
            .rooch
            .get_states(
                AccessPath::object(Account::account_object_id(account_address)),
                None,
            )
            .await?
            .pop()
            .flatten()
            .map(|state_view| {
                let state = ObjectState::from(state_view);
                state.into_object_uncheck::<Account>()
            })
            .transpose()?
            .map(|account| account.value);
        let bitcoin_address_opt = client.rooch.resolve_bitcoin_address(rooch_address).await?;
        let account_address_view = account_opt
            .clone()
            .map(|account| AccountAddressView::from(account.addr));
        let sequence_number_view = account_opt
            .clone()
            .map(|account| StrView::from(account.sequence_number));
        let bitcoin_address_view = bitcoin_address_opt.clone().map(BitcoinAddressView::from);
        let show_result_view = ShowResultView::new(
            account_address_view,
            sequence_number_view,
            bitcoin_address_view,
        );

        if self.json {
            Ok(Some(show_result_view))
        } else {
            // terminal
            let (width, height) = get_terminal_size();

            // account
            if account_opt.is_some() {
                // vectors
                let mut formatted_account_header = vec![];
                let mut formatted_account = vec![];
                let mut account_builder = Builder::default();

                formatted_account_header.push("Account Address".to_owned());
                formatted_account.push(account_opt.clone().unwrap().addr.to_canonical_string());
                formatted_account_header.push("Sequence Number".to_owned());
                formatted_account.push(account_opt.clone().unwrap().sequence_number.to_string());

                account_builder.push_record(formatted_account_header);
                account_builder.push_record(formatted_account);

                let mut account_table = account_builder.build();
                let styled_account_table = account_table
                    .with(Panel::header("Account"))
                    .with(Style::rounded())
                    .with(Width::wrap(width).priority(PriorityRight::new()))
                    .with(Width::increase(width))
                    .with(Height::limit(height))
                    .with(Height::increase(height))
                    .to_string();
                println!("{}", styled_account_table);
            }

            // bitcoin address
            if bitcoin_address_opt.is_some() {
                let mut formatted_bitcoin_address = vec![];
                let mut bitcoin_address_builder = Builder::default();
                formatted_bitcoin_address.push(bitcoin_address_opt.unwrap().to_string());
                bitcoin_address_builder.push_record(formatted_bitcoin_address);
                let mut bitcoin_address_table = bitcoin_address_builder.build();
                let styled_bitcoin_address_table = bitcoin_address_table
                    .with(Panel::header("Bitcoin Address"))
                    .with(Style::rounded())
                    .with(Width::wrap(width).priority(PriorityRight::new()))
                    .with(Width::increase(width))
                    .with(Height::limit(height))
                    .with(Height::increase(height))
                    .to_string();
                println!("{}", styled_bitcoin_address_table);
            }

            Ok(None)
        }
    }
}

fn get_terminal_size() -> (usize, usize) {
    if let Some((TerminalWidth(width), TerminalHeight(height))) = terminal_size() {
        (width as usize, height as usize)
    } else {
        // default
        (80, 24)
    }
}
