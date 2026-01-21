// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cli_types::{CommandAction, FileOrHexInput, WalletContextOptions},
    commands::bitcoin::retry_rpc_call,
};
use anyhow::Result;
use async_trait::async_trait;
use bitcoin::Psbt;
use clap::Parser;
use moveos_types::module_binding::MoveFunctionCaller;
use rooch_rpc_client::Client;
use rooch_types::{
    address::{BitcoinAddress, RoochAddress},
    bitcoin::multisign_account::MultisignAccountModule,
    error::RoochResult,
};
use std::collections::{HashMap, HashSet};
use tracing::debug;

#[derive(Debug, Parser)]
pub struct VerifyPsbt {
    /// The input psbt file path or hex string
    input: FileOrHexInput,

    /// Show detailed per-input status
    #[clap(long, default_value = "false")]
    verbose: bool,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[derive(Debug, Clone)]
struct InputVerificationStatus {
    input_index: usize,
    is_multisig: bool,
    threshold: Option<usize>,
    signatures_count: usize,
    has_final_witness: bool,
    is_satisfied: bool,
}

#[derive(Debug)]
struct PsbtVerificationResult {
    total_inputs: usize,
    multisig_inputs: usize,
    single_sig_inputs: usize,
    fully_signed_inputs: usize,
    satisfied_inputs: usize,
    input_statuses: Vec<InputVerificationStatus>,
    psbt: Psbt,
}

#[async_trait]
impl CommandAction<String> for VerifyPsbt {
    async fn execute(self) -> RoochResult<String> {
        let context = self.context_options.build()?;
        let client = context.get_client().await?;

        let psbt = Psbt::deserialize(&self.input.data)?;
        let result = verify_psbt(psbt, &client).await?;

        Ok(format_verification_result(&result, self.verbose))
    }
}

async fn verify_psbt(psbt: Psbt, client: &Client) -> Result<PsbtVerificationResult> {
    let multisign_account_module = client.as_module_binding::<MultisignAccountModule>();

    let total_inputs = psbt.inputs.len();
    let mut input_statuses = Vec::with_capacity(total_inputs);

    let mut multisig_inputs = 0;
    let mut single_sig_inputs = 0;
    let mut fully_signed_inputs = 0;
    let mut satisfied_inputs = 0;

    // Cache for multisign account info to avoid repeated RPC calls
    let mut multisign_account_cache: HashMap<
        RoochAddress,
        rooch_types::bitcoin::multisign_account::MultisignAccountInfo,
    > = HashMap::new();

    // Cache for is_multisign_account results
    let mut is_multisign_cache: HashSet<RoochAddress> = HashSet::new();
    let mut non_multisign_cache: HashSet<RoochAddress> = HashSet::new();

    for (idx, input) in psbt.inputs.iter().enumerate() {
        // Skip inputs without witness_utxo
        let utxo = match input.witness_utxo.as_ref() {
            Some(utxo) => utxo,
            None => {
                debug!("Skipping input {} (no witness_utxo)", idx);
                continue;
            }
        };

        let addr = BitcoinAddress::from(&utxo.script_pubkey);
        let rooch_addr = addr.to_rooch_address();

        // Check if this is a multisign account (use cache)
        let is_multisign = if is_multisign_cache.contains(&rooch_addr) {
            debug!("Using cached is_multisign result for {:?}", rooch_addr);
            true
        } else if non_multisign_cache.contains(&rooch_addr) {
            debug!("Using cached non-multisign result for {:?}", rooch_addr);
            false
        } else {
            debug!("Checking is_multisign_account for {:?}", rooch_addr);
            let is_ms = multisign_account_module.is_multisign_account(rooch_addr.into())?;
            if is_ms {
                is_multisign_cache.insert(rooch_addr.clone());
            } else {
                non_multisign_cache.insert(rooch_addr.clone());
            }
            is_ms
        };

        let (threshold, is_satisfied, has_final_witness) = if is_multisign {
            multisig_inputs += 1;

            // Fetch account info with retry (use cache if available)
            let account_info = if let Some(cached) = multisign_account_cache.get(&rooch_addr) {
                debug!("Using cached account info for {:?}", rooch_addr);
                cached.clone()
            } else {
                debug!("Fetching multisign account info for {:?}", rooch_addr);
                let info = retry_rpc_call(|| async {
                    client.rooch.get_multisign_account_info(rooch_addr).await
                })
                .await?;
                multisign_account_cache.insert(rooch_addr, info.clone());
                info
            };

            let threshold = account_info.threshold as usize;
            let signatures_count = input.tap_script_sigs.len();
            let is_satisfied = signatures_count >= threshold;
            let has_final_witness = input.final_script_witness.is_some();

            if has_final_witness {
                fully_signed_inputs += 1;
            }
            if is_satisfied {
                satisfied_inputs += 1;
            }

            (Some(threshold), is_satisfied, has_final_witness)
        } else {
            single_sig_inputs += 1;

            let has_final_witness = input.final_script_witness.is_some();
            if has_final_witness {
                fully_signed_inputs += 1;
                satisfied_inputs += 1;
            }

            (None, has_final_witness, has_final_witness)
        };

        input_statuses.push(InputVerificationStatus {
            input_index: idx,
            is_multisig: is_multisign,
            threshold,
            signatures_count: input.tap_script_sigs.len(),
            has_final_witness,
            is_satisfied,
        });
    }

    Ok(PsbtVerificationResult {
        total_inputs,
        multisig_inputs,
        single_sig_inputs,
        fully_signed_inputs,
        satisfied_inputs,
        input_statuses,
        psbt,
    })
}

fn format_verification_result(result: &PsbtVerificationResult, verbose: bool) -> String {
    let mut output = String::new();

    output.push_str("PSBT Verification Summary:\n");
    output.push_str(&format!("  Total inputs: {}\n", result.total_inputs));
    output.push_str(&format!("  Multi-sig inputs: {}\n", result.multisig_inputs));
    output.push_str(&format!(
        "  Single-sig inputs: {}\n",
        result.single_sig_inputs
    ));
    output.push_str(&format!(
        "  Fully signed (finalized): {}\n",
        result.fully_signed_inputs
    ));
    output.push_str(&format!(
        "  Threshold satisfied: {}\n",
        result.satisfied_input_count()
    ));

    // Add transaction outputs information
    if !result.psbt.unsigned_tx.output.is_empty() {
        output.push_str("\n");
        output.push_str("Transaction Outputs:\n");
        let mut total_sat = 0;
        for (i, tx_out) in result.psbt.unsigned_tx.output.iter().enumerate() {
            let amount_sat = tx_out.value.to_sat();
            total_sat += amount_sat;
            let amount_btc = amount_sat as f64 / 100_000_000.0;

            output.push_str(&format!("  Output {}:\n", i));
            output.push_str(&format!(
                "    Amount: {} satoshi ({:.8} BTC)\n",
                amount_sat, amount_btc
            ));

            // Try to decode address
            if let Ok(address) =
                bitcoin::Address::from_script(&tx_out.script_pubkey, bitcoin::Network::Bitcoin)
            {
                output.push_str(&format!("    Address: {}\n", address));
            } else {
                output.push_str(&format!("    Script: {}\n", tx_out.script_pubkey));
            }
        }
        output.push_str(&format!(
            "\n  Total output: {} satoshi ({:.8} BTC)\n",
            total_sat,
            total_sat as f64 / 100_000_000.0
        ));
    }

    output.push_str("\n");

    // Group by threshold for multisig inputs
    let mut threshold_groups: std::collections::HashMap<
        Option<usize>,
        Vec<&InputVerificationStatus>,
    > = std::collections::HashMap::new();

    for status in &result.input_statuses {
        threshold_groups
            .entry(status.threshold)
            .or_default()
            .push(status);
    }

    // Print multisig summary
    for (threshold, statuses) in &threshold_groups {
        if let Some(thresh) = threshold {
            let total_sigs: usize = statuses.iter().map(|s| s.signatures_count).sum();
            let avg_sigs = if statuses.is_empty() {
                0.0
            } else {
                total_sigs as f64 / statuses.len() as f64
            };
            output.push_str(&format!(
                "Multi-sig (threshold={}): {} inputs, {:.1} avg signatures\n",
                thresh,
                statuses.len(),
                avg_sigs
            ));

            // Estimate final transaction size
            if statuses.len() > 0 {
                let sig_size = 64; // Schnorr signature
                let script_size = 107; // Approximate multisig script for 7 participants
                let control_block_size = 65; // Taproot control block

                let witness_per_input =
                    thresh * (sig_size + 1) + script_size + 1 + control_block_size + 1;
                let base_tx_size = 35 + statuses.len() * 41 + 2 * 30; // Rough estimate
                let final_witness_size = witness_per_input * statuses.len();
                let final_tx_weight = base_tx_size * 4 + final_witness_size;

                output.push_str(&format!(
                    "  Estimated final size: {:.1} KB (weight: {} weight units)\n",
                    (base_tx_size + final_witness_size) as f64 / 1024.0,
                    final_tx_weight
                ));

                const MAX_STANDARD_WEIGHT: usize = 400_000;
                if final_tx_weight > MAX_STANDARD_WEIGHT {
                    let excess = final_tx_weight - MAX_STANDARD_WEIGHT;
                    let excess_pct = (excess as f64 / MAX_STANDARD_WEIGHT as f64) * 100.0;
                    output.push_str(&format!(
                        "  ⚠️  WARNING: Exceeds Bitcoin standard limit by {:.1}% ({} weight units)\n",
                        excess_pct, excess
                    ));
                    output.push_str(
                        "  This transaction cannot be broadcast as a standard transaction.\n",
                    );
                    output.push_str("  Consider splitting into multiple smaller transactions.\n");
                }
            }
        }
    }

    // Print detailed status for each input if verbose
    if verbose {
        output.push_str("\nDetailed Input Status:\n");

        // Sort by threshold and input index
        let mut sorted_statuses = result.input_statuses.clone();
        sorted_statuses.sort_by(|a, b| {
            a.threshold
                .cmp(&b.threshold)
                .then_with(|| a.input_index.cmp(&b.input_index))
        });

        for status in sorted_statuses {
            if status.is_multisig {
                let threshold = status.threshold.unwrap_or(0);
                let icon = if status.is_satisfied { "✓" } else { "✗" };
                output.push_str(&format!(
                    "  Input {}: {} signatures (threshold: {}) {} {}\n",
                    status.input_index,
                    status.signatures_count,
                    threshold,
                    icon,
                    if status.has_final_witness {
                        "[finalized]"
                    } else {
                        ""
                    }
                ));
            } else {
                let icon = if status.is_satisfied { "✓" } else { "✗" };
                output.push_str(&format!(
                    "  Input {}: single-sig {} {}\n",
                    status.input_index,
                    icon,
                    if status.has_final_witness {
                        "[finalized]"
                    } else {
                        ""
                    }
                ));
            }
        }
    }

    // Overall status
    output.push_str("\n");
    if result.fully_signed_inputs == result.total_inputs && result.total_inputs > 0 {
        output.push_str("Status: ✅ All inputs finalized - Ready to broadcast\n");
    } else if result.satisfied_inputs == result.total_inputs && result.total_inputs > 0 {
        output.push_str("Status: ⚠️  Threshold met but not finalized - run sign-tx to finalize\n");
    } else if result.satisfied_inputs > 0 {
        output.push_str(&format!(
            "Status: ⚠️  Partially signed ({} of {} inputs satisfied)\n",
            result.satisfied_input_count(),
            result.total_inputs
        ));
    } else {
        output.push_str("Status: ❌ No signatures - needs signing\n");
    }

    output
}

impl PsbtVerificationResult {
    fn satisfied_input_count(&self) -> usize {
        self.input_statuses
            .iter()
            .filter(|s| s.is_satisfied)
            .count()
    }
}
