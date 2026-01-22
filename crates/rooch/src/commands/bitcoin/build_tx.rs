// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::transaction_builder::TransactionBuilder;
use super::FileOutput;
use crate::cli_types::{CommandAction, WalletContextOptions};
use crate::commands::bitcoin::FileOutputData;
use async_trait::async_trait;
use bitcoin::absolute::LockTime;
use bitcoin::{Address, Amount, FeeRate, OutPoint};
use clap::Parser;
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_api::jsonrpc_types::btc::utxo::UTXOObjectView;
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::address::ParsedAddress;
use rooch_types::bitcoin::utxo::derive_utxo_id;
use rooch_types::error::{RoochError, RoochResult};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::debug;

#[derive(Debug, Clone)]
pub enum ParsedInput {
    ObjectID(ObjectID),
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
            ParsedInput::ObjectID(id) => id,
            ParsedInput::OutPoint(outpoint) => derive_utxo_id(&outpoint.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputAmount {
    All,
    Specific(Amount),
}

#[derive(Debug, Clone)]
pub struct ParsedOutput {
    pub address: ParsedAddress,
    pub amount: OutputAmount,
}

impl FromStr for ParsedOutput {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (addr_part, amount_part) = s
            .split_once(':')
            .ok_or_else(|| RoochError::CommandArgumentError("Invalid output format".to_string()))?;
        let address = ParsedAddress::parse(addr_part)?;

        if amount_part.eq_ignore_ascii_case("all") {
            return Ok(ParsedOutput {
                address,
                amount: OutputAmount::All,
            });
        }

        let amount = u64::from_str(amount_part)?;
        Ok(ParsedOutput {
            address,
            amount: OutputAmount::Specific(Amount::from_sat(amount)),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiFileOutput {
    pub files: Vec<FileInfo>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub output_type: String,
}

#[derive(Debug, Parser)]
pub struct BuildTx {
    #[clap(long, short = 's', value_parser=ParsedAddress::parse, default_value = "default")]
    sender: ParsedAddress,

    #[clap(long, short = 'i')]
    inputs: Vec<ParsedInput>,

    #[clap(long, short = 'o', required = true, num_args = 1..)]
    outputs: Vec<ParsedOutput>,

    #[clap(long)]
    fee_rate: Option<FeeRate>,

    #[clap(long)]
    lock_time: Option<LockTime>,

    #[clap(long, value_parser=ParsedAddress::parse)]
    change_address: Option<ParsedAddress>,

    #[clap(long)]
    skip_check_seal: bool,

    #[clap(long)]
    output_file: Option<String>,

    #[clap(long)]
    max_inputs: Option<usize>,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<String> for BuildTx {
    async fn execute(self) -> RoochResult<String> {
        let context = self.context_options.build_require_password()?;
        let client = context.get_client().await?;

        let bitcoin_network = context.get_bitcoin_network().await?;
        let sender_btc_addr = context.resolve_bitcoin_address(self.sender).await?;
        let sender = sender_btc_addr.to_bitcoin_address(bitcoin_network)?;

        let max_inputs = self.max_inputs.unwrap_or(usize::MAX);
        let send_all = self.outputs.iter().any(|o| o.amount == OutputAmount::All);

        // Validate :all usage - only one output can use :all
        if send_all {
            let all_count = self
                .outputs
                .iter()
                .filter(|o| o.amount == OutputAmount::All)
                .count();
            if all_count > 1 {
                return Err(RoochError::from(anyhow::anyhow!(
                    "Only one output can use :all. Found {} outputs with :all.",
                    all_count
                )));
            }
        }

        // Case 1: Manual inputs specified
        if !self.inputs.is_empty() {
            if send_all {
                return Err(RoochError::from(anyhow::anyhow!(
                    ":all cannot be used with manually specified --inputs. Remove --inputs to auto-load UTXOs."
                )));
            }

            let inputs: Vec<ObjectID> = self
                .inputs
                .into_iter()
                .map(|input| input.into_object_id())
                .collect();

            if inputs.len() > max_inputs {
                return Err(RoochError::from(anyhow::anyhow!(
                    "Too many inputs ({} > {}). Either reduce the number of inputs or remove --inputs to enable auto-splitting.",
                    inputs.len(),
                    max_inputs
                )));
            }

            // Build single transaction with manual inputs
            return build_single_transaction(
                &context,
                client,
                sender,
                bitcoin_network,
                inputs,
                self.skip_check_seal,
                self.fee_rate,
                self.lock_time,
                self.change_address,
                self.outputs,
                self.output_file,
            )
            .await;
        }

        // Case 2: Auto-loading UTXOs (no manual inputs)
        if send_all || self.max_inputs.is_some() {
            // Load all UTXOs first
            let mut temp_selector = crate::commands::bitcoin::utxo_selector::UTXOSelector::new(
                client.clone(),
                sender.clone(),
                vec![],
                self.skip_check_seal,
            )
            .await?;

            let all_utxos = temp_selector.load_all_utxos().await?;
            debug!("Loaded {} UTXOs", all_utxos.len());

            // Case 2a: Split into multiple transactions (max_inputs set)
            if all_utxos.len() > max_inputs {
                let chunks: Vec<Vec<UTXOObjectView>> =
                    all_utxos.chunks(max_inputs).map(|s| s.to_vec()).collect();

                debug!("Splitting into {} transactions", chunks.len());
                return build_multiple_transactions(
                    &context,
                    client,
                    sender,
                    bitcoin_network,
                    chunks,
                    self.fee_rate,
                    self.lock_time,
                    self.change_address,
                    self.outputs,
                    self.output_file,
                )
                .await;
            }

            // Case 2b: Single transaction with :all
            // Calculate total amount and let TransactionBuilder handle it
            if send_all {
                let total: Amount = all_utxos.iter().map(|u| u.amount()).sum();

                // Estimate fee
                let utxo_count = all_utxos.len();
                let estimated_vsize = 100 + utxo_count * 60 + 2 * 43;
                let fee_rate = self
                    .fee_rate
                    .unwrap_or_else(|| FeeRate::from_sat_per_vb(10).unwrap());
                let estimated_fee = fee_rate
                    .fee_vb(estimated_vsize as u64)
                    .unwrap_or(Amount::from_sat(estimated_vsize as u64));

                // Set output amount to total - fee (minus small margin)
                let output_amount = Amount::from_sat(
                    total
                        .to_sat()
                        .saturating_sub(estimated_fee.to_sat())
                        .saturating_sub(1000),
                );

                debug!(
                    "Send-all: total={}, output={}, fee={}",
                    total, output_amount, estimated_fee
                );

                // Convert :all to specific amount
                let specific_outputs = self
                    .outputs
                    .into_iter()
                    .map(|mut output| {
                        if output.amount == OutputAmount::All {
                            output.amount = OutputAmount::Specific(output_amount);
                        }
                        output
                    })
                    .collect::<Vec<_>>();

                // Build transaction with pre-loaded UTXOs to avoid redundant queries
                return build_single_transaction_with_utxos(
                    &context,
                    client,
                    sender,
                    bitcoin_network,
                    all_utxos,
                    self.skip_check_seal,
                    self.fee_rate,
                    self.lock_time,
                    self.change_address,
                    specific_outputs,
                    self.output_file,
                )
                .await;
            }

            // Case 2c: Single transaction with specific amount (but max_inputs was set)
            let specific_outputs = self.outputs.clone();
            return build_single_transaction(
                &context,
                client,
                sender,
                bitcoin_network,
                vec![], // Empty inputs triggers auto-loading
                self.skip_check_seal,
                self.fee_rate,
                self.lock_time,
                self.change_address,
                specific_outputs,
                self.output_file,
            )
            .await;
        }

        // Case 3: Normal single transaction (auto-load UTXOs)
        build_single_transaction(
            &context,
            client,
            sender,
            bitcoin_network,
            vec![], // Empty inputs triggers auto-loading
            self.skip_check_seal,
            self.fee_rate,
            self.lock_time,
            self.change_address,
            self.outputs,
            self.output_file,
        )
        .await
    }
}

async fn build_single_transaction(
    context: &WalletContext,
    client: rooch_rpc_client::Client,
    sender: Address,
    bitcoin_network: rooch_types::bitcoin::network::Network,
    inputs: Vec<ObjectID>,
    skip_check_seal: bool,
    fee_rate: Option<FeeRate>,
    lock_time: Option<LockTime>,
    change_address: Option<ParsedAddress>,
    outputs: Vec<ParsedOutput>,
    output_file: Option<String>,
) -> RoochResult<String> {
    let btc_network = bitcoin::Network::from(bitcoin_network);
    let mut tx_builder =
        TransactionBuilder::new(context, client, sender, inputs, skip_check_seal).await?;

    if let Some(fee_rate) = fee_rate {
        tx_builder = tx_builder.with_fee_rate(fee_rate);
    }
    if let Some(lock_time) = lock_time {
        tx_builder = tx_builder.with_lock_time(lock_time);
    }
    if let Some(change_address) = change_address {
        let change_btc_addr = context.resolve_bitcoin_address(change_address).await?;
        let change_address = change_btc_addr.to_bitcoin_address(btc_network)?;
        tx_builder = tx_builder.with_change_address(change_address);
    }

    let mut converted_outputs = Vec::new();
    for output in outputs {
        let btc_address = context.resolve_bitcoin_address(output.address).await?;
        let address = btc_address.to_bitcoin_address(btc_network)?;
        let amount = match output.amount {
            OutputAmount::All => {
                // This shouldn't happen here, :all should be handled before calling this function
                return Err(RoochError::from(anyhow::anyhow!(
                    "Unexpected :all in build_single_transaction"
                )));
            }
            OutputAmount::Specific(amt) => amt,
        };
        converted_outputs.push((address, amount));
    }

    let psbt = tx_builder.build(converted_outputs).await?;
    debug!("PSBT built successfully");

    let fileout = FileOutput::write_to_file(FileOutputData::Psbt(psbt), output_file)?;
    Ok(serde_json::to_string_pretty(&fileout).unwrap())
}

async fn build_multiple_transactions(
    context: &WalletContext,
    client: rooch_rpc_client::Client,
    sender: Address,
    bitcoin_network: rooch_types::bitcoin::network::Network,
    chunks: Vec<Vec<UTXOObjectView>>,
    fee_rate: Option<FeeRate>,
    lock_time: Option<LockTime>,
    change_address: Option<ParsedAddress>,
    outputs: Vec<ParsedOutput>,
    output_file: Option<String>,
) -> RoochResult<String> {
    let btc_network = bitcoin::Network::from(bitcoin_network);

    // Resolve addresses and collect output amount types
    let mut resolved_outputs = Vec::new();
    for output in &outputs {
        let btc_address = context
            .resolve_bitcoin_address(output.address.clone())
            .await?;
        let address = btc_address.to_bitcoin_address(btc_network)?;
        resolved_outputs.push((address, output.amount.clone()));
    }

    let mut all_files = Vec::new();
    let total_chunks = chunks.len();

    for (chunk_idx, chunk_utxos) in chunks.into_iter().enumerate() {
        // Calculate output amount for this chunk if using :all
        let chunk_outputs: Vec<(Address, Amount)> = resolved_outputs
            .iter()
            .map(|(addr, amount_type)| {
                let amount = match amount_type {
                    OutputAmount::All => {
                        let chunk_total: Amount = chunk_utxos.iter().map(|u| u.amount()).sum();

                        // Estimate fee
                        let utxo_count = chunk_utxos.len();
                        let estimated_vsize = 100 + utxo_count * 60 + 2 * 43;
                        let fee_rate_val =
                            fee_rate.unwrap_or_else(|| FeeRate::from_sat_per_vb(10).unwrap());
                        let estimated_fee = fee_rate_val
                            .fee_vb(estimated_vsize as u64)
                            .unwrap_or(Amount::from_sat(estimated_vsize as u64));

                        let output_amount = Amount::from_sat(
                            chunk_total
                                .to_sat()
                                .saturating_sub(estimated_fee.to_sat())
                                .saturating_sub(1000),
                        );

                        debug!(
                            "Transaction {}: total={}, output={}, fee={}",
                            chunk_idx + 1,
                            chunk_total,
                            output_amount,
                            estimated_fee
                        );

                        output_amount
                    }
                    OutputAmount::Specific(amt) => *amt,
                };

                (addr.clone(), amount)
            })
            .collect();

        // Use pre-loaded UTXOs directly, avoiding redundant queries
        let mut tx_builder =
            TransactionBuilder::with_utxos(context, client.clone(), sender.clone(), chunk_utxos);

        if let Some(fee_rate_val) = fee_rate {
            tx_builder = tx_builder.with_fee_rate(fee_rate_val);
        }
        if let Some(lock_time) = lock_time {
            tx_builder = tx_builder.with_lock_time(lock_time);
        }
        if let Some(ref change_addr) = change_address {
            let change_btc_addr = context.resolve_bitcoin_address(change_addr.clone()).await?;
            let change_address = change_btc_addr.to_bitcoin_address(btc_network)?;
            tx_builder = tx_builder.with_change_address(change_address);
        }

        let psbt = tx_builder.build(chunk_outputs).await?;
        debug!("Transaction {} PSBT built successfully", chunk_idx + 1);

        let chunk_output_file = output_file.as_ref().map(|base_path| {
            if let Some(ext_pos) = base_path.rfind('.') {
                format!(
                    "{}_{}{}",
                    &base_path[..ext_pos],
                    chunk_idx + 1,
                    &base_path[ext_pos..]
                )
            } else {
                format!("{}_{}", base_path, chunk_idx + 1)
            }
        });

        let fileout = FileOutput::write_to_file(FileOutputData::Psbt(psbt), chunk_output_file)?;
        all_files.push(FileInfo {
            path: fileout.path,
            output_type: fileout.output_type,
        });
    }

    let result = MultiFileOutput {
        files: all_files,
        count: total_chunks,
    };
    Ok(serde_json::to_string_pretty(&result).unwrap())
}

async fn build_single_transaction_with_utxos(
    context: &WalletContext,
    client: rooch_rpc_client::Client,
    sender: Address,
    bitcoin_network: rooch_types::bitcoin::network::Network,
    utxos: Vec<UTXOObjectView>,
    _skip_check_seal: bool, // UTXOs already checked during load_all_utxos
    fee_rate: Option<FeeRate>,
    lock_time: Option<LockTime>,
    change_address: Option<ParsedAddress>,
    outputs: Vec<ParsedOutput>,
    output_file: Option<String>,
) -> RoochResult<String> {
    let btc_network = bitcoin::Network::from(bitcoin_network);

    // Use TransactionBuilder with pre-loaded UTXOs to avoid redundant queries
    let mut tx_builder = TransactionBuilder::with_utxos(context, client.clone(), sender, utxos);

    if let Some(fee_rate) = fee_rate {
        tx_builder = tx_builder.with_fee_rate(fee_rate);
    }
    if let Some(lock_time) = lock_time {
        tx_builder = tx_builder.with_lock_time(lock_time);
    }
    if let Some(change_address) = change_address {
        let change_btc_addr = context.resolve_bitcoin_address(change_address).await?;
        let change_address = change_btc_addr.to_bitcoin_address(btc_network)?;
        tx_builder = tx_builder.with_change_address(change_address);
    }

    let mut converted_outputs = Vec::new();
    for output in outputs {
        let btc_address = context.resolve_bitcoin_address(output.address).await?;
        let address = btc_address.to_bitcoin_address(btc_network)?;
        let amount = match output.amount {
            OutputAmount::All => {
                // This shouldn't happen here, :all should be handled before calling this function
                return Err(RoochError::from(anyhow::anyhow!(
                    "Unexpected :all in build_single_transaction_with_utxos"
                )));
            }
            OutputAmount::Specific(amt) => amt,
        };
        converted_outputs.push((address, amount));
    }

    let psbt = tx_builder.build(converted_outputs).await?;
    debug!("PSBT built successfully");

    let fileout = FileOutput::write_to_file(FileOutputData::Psbt(psbt), output_file)?;
    Ok(serde_json::to_string_pretty(&fileout).unwrap())
}
