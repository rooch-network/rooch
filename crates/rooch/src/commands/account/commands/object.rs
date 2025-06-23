// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use bitcoin::hex::DisplayHex;
use clap::Parser;
use rooch_rpc_api::jsonrpc_types::{
    IndexerObjectStateView, ObjectIDView, ObjectStateFilterView, RoochAddressView,
    UnitedAddressView,
};
use rooch_types::{address::ParsedAddress, error::RoochResult};
use std::collections::HashMap;
use tabled::{
    builder::Builder,
    settings::{peaker::PriorityRight, Height, Panel, Style, Width},
};
use terminal_size::{terminal_size, Height as TerminalHeight, Width as TerminalWidth};

/// List objects of a holding account on Rooch Network. Requires internet connection and works without rooch init.
#[derive(Debug, Parser)]
pub struct ObjectCommand {
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

pub type ObjectResultView = HashMap<ObjectIDView, IndexerObjectStateView>;

#[async_trait]
impl CommandAction<Option<ObjectResultView>> for ObjectCommand {
    async fn execute(self) -> RoochResult<Option<ObjectResultView>> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;
        let mapping = context.address_mapping();
        let rooch_address = self.address.into_rooch_address(&mapping)?;
        let rooch_address_view = RoochAddressView::from(rooch_address);

        // object states
        let united_address_view = UnitedAddressView::from(rooch_address_view);
        let object_state_filter_view = ObjectStateFilterView::Owner(united_address_view.clone());
        let object_states = client
            .rooch
            .query_object_states(object_state_filter_view, None, Some(self.limit), None)
            .await?
            .data;

        // object result view with object id view and indexer object state view
        let mut object_result_view: HashMap<ObjectIDView, IndexerObjectStateView> = HashMap::new();
        for object_state in &object_states {
            let key = object_state.metadata.id.clone();
            object_result_view.insert(key.into(), object_state.clone());
        }

        if self.json {
            Ok(Some(object_result_view))
        } else {
            let mut formatted_indexer_object_state_header = vec![];
            let mut formatted_indexer_object_state = vec![];

            // terminal
            let (width, height) = get_terminal_size();

            // objects
            let mut indexer_object_state_builder = Builder::default();
            formatted_indexer_object_state_header.push("Id".to_owned());
            formatted_indexer_object_state_header.push("Owner".to_owned());
            formatted_indexer_object_state_header.push("Owner Bitcoin Address".to_owned());
            formatted_indexer_object_state_header.push("Flag".to_owned());
            formatted_indexer_object_state_header.push("State Root".to_owned());
            formatted_indexer_object_state_header.push("Size".to_owned());
            formatted_indexer_object_state_header.push("Created At".to_owned());
            formatted_indexer_object_state_header.push("Updated At".to_owned());
            formatted_indexer_object_state_header.push("Object Type".to_owned());
            formatted_indexer_object_state_header.push("Value".to_owned());
            formatted_indexer_object_state_header.push("Decoded Value".to_owned());
            formatted_indexer_object_state_header.push("Tx Order".to_owned());
            formatted_indexer_object_state_header.push("State Index".to_owned());
            formatted_indexer_object_state_header.push("Display Fields".to_owned());
            for object_state in object_states {
                formatted_indexer_object_state.push(object_state.metadata.id.to_hex());
                formatted_indexer_object_state.push(object_state.metadata.owner.0.to_bech32());
                formatted_indexer_object_state.push(
                    object_state
                        .metadata
                        .owner_bitcoin_address
                        .unwrap_or_default(),
                );
                formatted_indexer_object_state.push(object_state.metadata.flag.to_string());
                formatted_indexer_object_state.push(
                    object_state
                        .metadata
                        .state_root
                        .unwrap_or_default()
                        .0
                        .to_string(),
                );
                formatted_indexer_object_state.push(object_state.metadata.size.0.to_string());
                formatted_indexer_object_state.push(object_state.metadata.created_at.0.to_string());
                formatted_indexer_object_state.push(object_state.metadata.updated_at.0.to_string());
                formatted_indexer_object_state
                    .push(object_state.metadata.object_type.0.to_canonical_string());
                formatted_indexer_object_state.push(object_state.value.0.as_hex().to_string());
                formatted_indexer_object_state.push(
                    object_state
                        .decoded_value
                        .unwrap_or_default()
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                );
                formatted_indexer_object_state.push(object_state.indexer_id.tx_order.0.to_string());
                formatted_indexer_object_state
                    .push(object_state.indexer_id.state_index.0.to_string());
                if object_state.display_fields.is_none() {
                    formatted_indexer_object_state.push("".to_owned());
                } else {
                    let display_fields = object_state.display_fields.unwrap();
                    let mut display_fields_strings = String::new();
                    for field in display_fields.fields {
                        display_fields_strings.push_str(field.0.as_str());
                        display_fields_strings.push_str(field.1.as_str());
                    }
                    formatted_indexer_object_state.push(display_fields_strings);
                }
            }
            indexer_object_state_builder.push_record(formatted_indexer_object_state_header);
            indexer_object_state_builder.push_record(formatted_indexer_object_state);
            let mut indexer_object_state_table = indexer_object_state_builder.build();
            let styled_indexer_object_state_table = indexer_object_state_table
                .with(Panel::header("Objects"))
                .with(Style::rounded())
                .with(Width::wrap(width).priority(PriorityRight::new()))
                .with(Width::increase(width))
                .with(Height::limit(height))
                .with(Height::increase(height))
                .to_string();
            println!("{}", styled_indexer_object_state_table);

            Ok(None)
        }
    }
}

fn get_terminal_size() -> (usize, usize) {
    if let Some((TerminalWidth(width), TerminalHeight(height))) = terminal_size() {
        (width as usize, height as usize)
    } else {
        // default terminal size
        (80, 24)
    }
}
