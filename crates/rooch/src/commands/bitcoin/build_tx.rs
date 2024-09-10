// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::transaction_builder::TransactionBuilder;
use super::FileOutput;
use crate::cli_types::{CommandAction, WalletContextOptions};
use crate::commands::bitcoin::FileOutputData;
use async_trait::async_trait;
use bitcoin::absolute::LockTime;
use bitcoin::{Amount, FeeRate, OutPoint};
use clap::Parser;
use moveos_types::moveos_std::object::ObjectID;
use rooch_types::address::ParsedAddress;
use rooch_types::bitcoin::utxo::derive_utxo_id;
use rooch_types::error::{RoochError, RoochResult};
use std::str::FromStr;
use tracing::debug;

#[derive(Debug, Clone)]
pub enum ParsedInput {
    /// Input is an UTXO ObjectID
    ObjectID(ObjectID),
    /// Input is an OutPoint
    OutPoint(OutPoint),
}

impl FromStr for ParsedInput {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(':') {
            let outpoint = OutPoint::from_str(s)?;
            Ok(ParsedInput::OutPoint(outpoint))
        } else {
            let object_id = ObjectID::from_str(s)?;
            Ok(ParsedInput::ObjectID(object_id))
        }
    }
}

impl ParsedInput {
    pub fn into_object_id(self) -> ObjectID {
        match self {
            ParsedInput::ObjectID(object_id) => object_id,
            ParsedInput::OutPoint(outpoint) => derive_utxo_id(&outpoint.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedOutput {
    pub address: ParsedAddress,
    pub amount: Amount,
}

impl FromStr for ParsedOutput {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (addr_part, amount_part) = s
            .split_once(':')
            .ok_or_else(|| RoochError::CommandArgumentError("Invalid output format".to_string()))?;
        let address = ParsedAddress::parse(addr_part)?;
        let amount = u64::from_str(amount_part)?;
        Ok(ParsedOutput {
            address,
            amount: Amount::from_sat(amount),
        })
    }
}

#[derive(Debug, Parser)]
pub struct BuildTx {
    /// The sender address of the transaction, if not specified, the active address will be used
    #[clap(long, short = 's', value_parser=ParsedAddress::parse, default_value = "default")]
    sender: ParsedAddress,

    /// The inputs of the transaction, if not specified, the UTXOs of the sender will be used
    /// The format of the input is <object_id> or <txid>:<vout>
    #[clap(long, short = 'i')]
    inputs: Vec<ParsedInput>,

    // /// The to address of the transaction, if not specified, the outputs will be used
    // #[clap(long, short = 't', conflicts_with = "outputs", group = "to", value_parser=ParsedAddress::parse)]
    // to: Option<ParsedAddress>,

    // /// The amount of the transaction, if not specified, the amount will be calculated automatically
    // #[clap(long, short = 'a', conflicts_with = "outputs", group = "to")]
    // amount: Option<Amount>,
    #[clap(long, short = 'o', required = true, num_args = 1..)]
    outputs: Vec<ParsedOutput>,

    /// The fee rate of the transaction, if not specified, the fee will be calculated automatically
    #[clap(long)]
    fee_rate: Option<FeeRate>,

    /// The lock time of the transaction, if not specified, the lock time will be 0
    #[clap(long)]
    lock_time: Option<LockTime>,

    /// The change address of the transaction, if not specified, the change address will be the sender's address
    #[clap(long, value_parser=ParsedAddress::parse)]
    change_address: Option<ParsedAddress>,

    /// Skip check seal of the UTXOs, default is false
    /// If set to true, some UTXO which carries other asserts, such as Inscription, maybe unexpected spent.
    #[clap(long)]
    skip_check_seal: bool,

    /// The output file path for the psbt
    /// If not specified, the output will write to temp directory.
    #[clap(long)]
    output_file: Option<String>,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<FileOutput> for BuildTx {
    async fn execute(self) -> RoochResult<FileOutput> {
        let context = self.context_options.build_require_password()?;
        let client = context.get_client().await?;

        let bitcoin_network = context.get_bitcoin_network().await?;

        let sender = context.resolve_bitcoin_address(self.sender).await?;

        let inputs = self
            .inputs
            .into_iter()
            .map(|input| input.into_object_id())
            .collect();
        let mut tx_builder = TransactionBuilder::new(
            &context,
            client,
            sender.to_bitcoin_address(bitcoin_network)?,
            inputs,
            self.skip_check_seal,
        )
        .await?;

        if let Some(fee_rate) = self.fee_rate {
            tx_builder = tx_builder.with_fee_rate(fee_rate);
        }
        if let Some(lock_time) = self.lock_time {
            tx_builder = tx_builder.with_lock_time(lock_time);
        }
        if let Some(change_address) = self.change_address {
            let change_address = context.resolve_bitcoin_address(change_address).await?;
            tx_builder =
                tx_builder.with_change_address(change_address.to_bitcoin_address(bitcoin_network)?);
        }

        let mut outputs = Vec::new();
        for output in self.outputs.iter() {
            let address = context
                .resolve_bitcoin_address(output.address.clone())
                .await?;
            outputs.push((address.to_bitcoin_address(bitcoin_network)?, output.amount));
        }
        let psbt = tx_builder.build(outputs).await?;
        debug!("PSBT: {}", serde_json::to_string_pretty(&psbt).unwrap());
        let fileout = FileOutput::write_to_file(FileOutputData::Psbt(psbt), self.output_file)?;
        Ok(fileout)
    }
}
