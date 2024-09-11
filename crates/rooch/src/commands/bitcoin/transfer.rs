// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::sign_tx::SignOutput;
use super::transaction_builder::TransactionBuilder;
use crate::cli_types::{CommandAction, WalletContextOptions};
use crate::commands::bitcoin::sign_tx::sign_psbt;
use async_trait::async_trait;
use bitcoin::consensus::Encodable;
use bitcoin::{Amount, FeeRate};
use clap::Parser;
use rooch_types::address::ParsedAddress;
use rooch_types::error::{RoochError, RoochResult};
use tracing::debug;

#[derive(Debug, Parser)]
pub struct Transfer {
    /// The sender address of the transaction, if not specified, the active address will be used
    #[clap(long, short = 's', default_value = "default")]
    sender: ParsedAddress,

    /// The receiver address of the BTC
    #[clap(long, short = 't')]
    to: ParsedAddress,

    /// The BTC amount in satoshi to transfer
    #[clap(long, short = 'a')]
    amount: u64,

    /// The fee rate of the transaction, if not specified, the fee will be calculated automatically
    #[clap(long)]
    fee_rate: Option<FeeRate>,

    /// Skip check seal of the UTXOs, default is false
    /// If set to true, some UTXO which carries other asserts, such as Inscription, maybe unexpected spent.
    #[clap(long)]
    skip_check_seal: bool,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<String> for Transfer {
    async fn execute(self) -> RoochResult<String> {
        let context = self.context_options.build_require_password()?;
        let client = context.get_client().await?;

        let bitcoin_network = context.get_bitcoin_network().await?;

        let sender = context.resolve_bitcoin_address(self.sender).await?;
        let to = context.resolve_bitcoin_address(self.to).await?;
        let amount = Amount::from_sat(self.amount);

        let mut tx_builder = TransactionBuilder::new(
            &context,
            client.clone(),
            sender.to_bitcoin_address(bitcoin_network)?,
            vec![],
            self.skip_check_seal,
        )
        .await?;

        if let Some(fee_rate) = self.fee_rate {
            tx_builder = tx_builder.with_fee_rate(fee_rate);
        }

        let psbt = tx_builder
            .build_transfer(to.to_bitcoin_address(bitcoin_network)?, amount)
            .await?;
        debug!("PSBT: {}", serde_json::to_string_pretty(&psbt).unwrap());
        let sign_out = sign_psbt(psbt, None, &context, &client).await?;
        match sign_out {
            SignOutput::Psbt(_psbt) => {
                return Err(RoochError::CommandArgumentError(
                    "The sender address should not be a multisig address".to_string(),
                ))
            }
            SignOutput::Tx(tx) => {
                let mut raw_tx = vec![];
                tx.consensus_encode(&mut raw_tx)?;
                Ok(client
                    .rooch
                    .broadcast_bitcoin_tx(raw_tx.into(), None)
                    .await?)
            }
        }
    }
}
