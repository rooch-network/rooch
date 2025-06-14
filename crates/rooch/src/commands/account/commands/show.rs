// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_rpc_api::jsonrpc_types::{BitcoinAddressView, RoochAddressView};
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

/// Show account info on Rooch Network. Requires internet connection and works without rooch init.
#[derive(Debug, Parser)]
pub struct ShowCommand {
    #[clap(short = 'a', long = "address", value_parser=ParsedAddress::parse, default_value = "default")]
    address: ParsedAddress,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    #[clap(short = 'l', long = "limit", default_value = "50")]
    limit: u64,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct AccountInfoView {
    pub address: RoochAddressView,
    pub bitcoin_address: Option<BitcoinAddressView>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct RoochNetworkAccountView {
    pub account: AccountInfoView,
}

impl RoochNetworkAccountView {
    pub fn from_account(
        address: RoochAddressView,
        bitcoin_address: Option<BitcoinAddressView>,
    ) -> Self {
        // account info view with rooch address
        let account_info_view = AccountInfoView {
            address,
            bitcoin_address,
        };
        RoochNetworkAccountView {
            account: account_info_view,
        }
    }
}

#[async_trait]
impl CommandAction<Option<RoochNetworkAccountView>> for ShowCommand {
    async fn execute(self) -> RoochResult<Option<RoochNetworkAccountView>> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;
        let mapping = context.address_mapping();
        let rooch_address = self.address.into_rooch_address(&mapping)?;
        let rooch_address_view = RoochAddressView::from(rooch_address);
        let bitcoin_address_opt = client.rooch.resolve_bitcoin_address(rooch_address).await?;
        // rooch network account info from input address
        let rooch_network_account_view = if bitcoin_address_opt.clone().is_some() {
            let bitcoin_address = bitcoin_address_opt.clone().unwrap();
            let bitcoin_address_view = BitcoinAddressView::from(bitcoin_address);
            RoochNetworkAccountView::from_account(rooch_address_view, Some(bitcoin_address_view))
        } else {
            RoochNetworkAccountView::from_account(rooch_address_view, None)
        };

        if self.json {
            Ok(Some(rooch_network_account_view))
        } else {
            // vectors
            let mut formatted_account_info_header = vec![];
            let mut formatted_account_info = vec![];

            // terminal
            let (width, height) = get_terminal_size();

            // account
            let mut account_info_builder = Builder::default();
            formatted_account_info_header.push("Rooch Address".to_owned());
            formatted_account_info.push(rooch_address.to_bech32());
            if bitcoin_address_opt.clone().is_some() {
                formatted_account_info_header.push("Bitcoin Address".to_owned());
                formatted_account_info.push(bitcoin_address_opt.unwrap().to_string());
            }
            account_info_builder.push_record(formatted_account_info_header);
            account_info_builder.push_record(formatted_account_info);
            let mut account_info_table = account_info_builder.build();
            account_info_table
                .with(Panel::header("Account Info"))
                .with(Style::rounded())
                .with(Width::wrap(width).priority(PriorityRight::new()))
                .with(Width::increase(width))
                .with(Height::limit(height))
                .with(Height::increase(height))
                .to_string();

            println!("{}", account_info_table);

            Ok(None)
        }
    }
}

fn get_terminal_size() -> (usize, usize) {
    let (TerminalWidth(width), TerminalHeight(height)) =
        terminal_size().expect("failed to obtain a terminal size");

    (width as usize, height as usize)
}
