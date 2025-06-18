// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
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
    pub account_address: AccountAddressView,
    pub sequence_number: StrView<u64>,
    pub bitcoin_address: Option<BitcoinAddressView>,
}

impl ShowResultView {
    pub fn new(
        account_address: AccountAddressView,
        sequence_number: StrView<u64>,
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
        let rooch_address = self.address.into_rooch_address(&mapping)?;
        let account_states = client.rooch.get_account_states(rooch_address).await?;
        let bitcoin_address_opt = client.rooch.resolve_bitcoin_address(rooch_address).await?;
        let show_result_view = if bitcoin_address_opt.clone().is_some() {
            let bitcoin_address = bitcoin_address_opt.clone().unwrap();
            let bitcoin_address_view = BitcoinAddressView::from(bitcoin_address);
            ShowResultView::new(
                account_states.0.addr.into(),
                account_states.0.sequence_number.into(),
                Some(bitcoin_address_view),
            )
        } else {
            ShowResultView::new(
                account_states.0.addr.into(),
                account_states.0.sequence_number.into(),
                None,
            )
        };

        if self.json {
            Ok(Some(show_result_view))
        } else {
            // vectors
            let mut formatted_account_header = vec![];
            let mut formatted_account = vec![];

            // terminal
            let (width, height) = get_terminal_size();

            // account
            let mut account_builder = Builder::default();
            formatted_account_header.push("Account Address".to_owned());
            formatted_account.push(account_states.0.addr.to_canonical_string());
            formatted_account_header.push("Sequence Number".to_owned());
            formatted_account.push(account_states.0.sequence_number.to_string());
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

            // bitcoin address
            if bitcoin_address_opt.clone().is_some() {
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
