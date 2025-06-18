// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_rpc_api::jsonrpc_types::{BitcoinAddressView, NetworkAddressView};
use rooch_types::address::NetworkAddress;
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

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct NetworkAccountView {
    pub network_address: NetworkAddressView,
    pub bitcoin_address: Option<BitcoinAddressView>,
}

impl NetworkAccountView {
    pub fn new(
        network_address: NetworkAddressView,
        bitcoin_address: Option<BitcoinAddressView>,
    ) -> Self {
        // network account view with network address (rooch address and sequence number) and bitcoin address
        NetworkAccountView {
            network_address,
            bitcoin_address,
        }
    }
}

#[async_trait]
impl CommandAction<Option<NetworkAccountView>> for ShowCommand {
    async fn execute(self) -> RoochResult<Option<NetworkAccountView>> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;
        let mapping = context.address_mapping();
        let rooch_address = self.address.clone().into_rooch_address(&mapping)?;
        let sequence_number = client.rooch.get_sequence_number(rooch_address).await?;
        let network_address = NetworkAddress::new(rooch_address, sequence_number);
        let network_address_view = NetworkAddressView::from(network_address);
        let bitcoin_address_opt = client.rooch.resolve_bitcoin_address(rooch_address).await?;
        // rooch network account info from input address
        let network_account_view = if bitcoin_address_opt.clone().is_some() {
            let bitcoin_address = bitcoin_address_opt.clone().unwrap();
            let bitcoin_address_view = BitcoinAddressView::from(bitcoin_address);
            NetworkAccountView::new(network_address_view, Some(bitcoin_address_view))
        } else {
            NetworkAccountView::new(network_address_view, None)
        };

        if self.json {
            Ok(Some(network_account_view))
        } else {
            // vectors
            let mut formatted_network_address_header = vec![];
            let mut formatted_network_address = vec![];

            // terminal
            let (width, height) = get_terminal_size();

            // network address
            let mut network_address_builder = Builder::default();
            formatted_network_address_header.push("Rooch Address".to_owned());
            formatted_network_address.push(rooch_address.to_bech32());
            formatted_network_address_header.push("Sequence Number".to_owned());
            formatted_network_address.push(sequence_number.to_string());
            network_address_builder.push_record(formatted_network_address_header);
            network_address_builder.push_record(formatted_network_address);
            let mut network_address_table = network_address_builder.build();
            network_address_table
                .with(Panel::header("Network Address"))
                .with(Style::rounded())
                .with(Width::wrap(width).priority(PriorityRight::new()))
                .with(Width::increase(width))
                .with(Height::limit(height))
                .with(Height::increase(height))
                .to_string();

            println!("{}", network_address_table);

            // bitcoin address
            if bitcoin_address_opt.clone().is_some() {
                let mut formatted_bitcoin_address = vec![];
                let mut bitcoin_address_builder = Builder::default();
                formatted_bitcoin_address.push(bitcoin_address_opt.unwrap().to_string());
                bitcoin_address_builder.push_record(formatted_bitcoin_address);
                let mut bitcoin_address_table = bitcoin_address_builder.build();
                bitcoin_address_table
                    .with(Panel::header("Bitcoin Address"))
                    .with(Style::rounded())
                    .with(Width::wrap(width).priority(PriorityRight::new()))
                    .with(Width::increase(width))
                    .with(Height::limit(height))
                    .with(Height::increase(height))
                    .to_string();
                println!("{}", bitcoin_address_table);
            }

            Ok(None)
        }
    }
}

fn get_terminal_size() -> (usize, usize) {
    let (TerminalWidth(width), TerminalHeight(height)) =
        terminal_size().expect("failed to obtain a terminal size");

    (width as usize, height as usize)
}
