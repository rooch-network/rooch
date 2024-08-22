// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use async_trait::async_trait;
use rooch_types::{
    error::{RoochError, RoochResult},
    transaction::RoochTransactionData,
};
use std::{
    fs::File,
    io::{Read, Write},
};

/// Get transactions by order
#[derive(Debug, clap::Parser)]
pub struct SignCommand {
    /// Transaction data hex to be used for signing
    #[clap(long)]
    tx_hex: Option<String>,

    #[clap(flatten)]
    tx_options: TransactionOptions,

    #[clap(flatten)]
    context: WalletContextOptions,

    /// File location for the file being read
    #[clap(long)]
    file_location: Option<String>,

    /// File destination for the file being written
    #[clap(long)]
    file_destination: Option<String>,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<String>> for SignCommand {
    async fn execute(self) -> RoochResult<Option<String>> {
        let context = self.context.build()?;
        let password = context.get_password();
        let sender = context.resolve_address(self.tx_options.sender)?.into();
        let max_gas_amount = self.tx_options.max_gas_amount;
        let signed_tx;

        if let Some(file_location) = self.file_location {
            let mut file = File::open(file_location)?;
            let mut encoded_tx_data = Vec::new();
            file.read_to_end(&mut encoded_tx_data)?;
            let tx_data: RoochTransactionData =
                bcs::from_bytes(&encoded_tx_data).map_err(|_| {
                    RoochError::BcsError(format!("Invalid encoded tx data: {:?}", encoded_tx_data))
                })?;
            signed_tx = context
                .sign(sender, tx_data.action, password, max_gas_amount)
                .await?;
        } else if let Some(tx_hex) = self.tx_hex {
            let encoded_tx_data = hex::decode(tx_hex.clone()).map_err(|_| {
                RoochError::CommandArgumentError(format!("Invalid transaction hex: {}", tx_hex))
            })?;
            let tx_data: RoochTransactionData =
                bcs::from_bytes(&encoded_tx_data).map_err(|_| {
                    RoochError::BcsError(format!("Invalid encoded tx data: {:?}", encoded_tx_data))
                })?;
            signed_tx = context
                .sign(sender, tx_data.action, password, max_gas_amount)
                .await?;
        } else {
            return Err(RoochError::CommandArgumentError(
                "Argument --file-location or --tx-hex are not provided".to_owned(),
            ));
        }

        if let Some(file_destination) = self.file_destination {
            let mut file = File::create(file_destination)?;
            file.write_all(&signed_tx.encode())?;
            println!("Write signed tx data succeeded in the destination");

            Ok(None)
        } else {
            let signed_tx_hex = hex::encode(signed_tx.encode());
            if self.json {
                Ok(Some(signed_tx_hex))
            } else {
                println!(
                    "Sign transaction succeeded with the signed transaction hex [{}]",
                    signed_tx_hex
                );

                Ok(None)
            }
        }
    }
}
