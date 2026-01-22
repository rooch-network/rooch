// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::utxo_selector::UTXOSelector;
use anyhow::{anyhow, bail, Result};
use bitcoin::{
    absolute::LockTime, bip32::Fingerprint, transaction::Version, Address, Amount, FeeRate,
    OutPoint, Psbt, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness,
};
use moveos_types::{module_binding::MoveFunctionCaller, moveos_std::object::ObjectID};
use rooch_rpc_api::jsonrpc_types::btc::utxo::UTXOObjectView;
use rooch_rpc_client::{wallet_context::WalletContext, Client};
use rooch_types::address::RoochAddress;
use rooch_types::bitcoin::multisign_account::{self, PrecomputedMultisigPsbtData};
use tracing::debug;

// Use retry configuration and functions from mod.rs
use super::{is_rate_limit_error, retry_rpc_call, MAX_RETRIES, RETRY_DELAY};

#[derive(Debug)]
pub struct TransactionBuilder<'a> {
    wallet_context: &'a WalletContext,
    client: Client,
    utxo_selector: UTXOSelector,
    fee_rate: FeeRate,
    change_address: Address,
    lock_time: Option<LockTime>,
}

impl<'a> TransactionBuilder<'a> {
    const ADDITIONAL_INPUT_VBYTES: usize = 58;
    const ADDITIONAL_OUTPUT_VBYTES: usize = 43;
    const SCHNORR_SIGNATURE_SIZE: usize = 64;

    pub async fn new(
        wallet_context: &'a WalletContext,
        client: Client,
        sender: Address,
        inputs: Vec<ObjectID>,
        skip_seal_check: bool,
    ) -> Result<Self> {
        let utxo_selector =
            UTXOSelector::new(client.clone(), sender.clone(), inputs, skip_seal_check).await?;
        Ok(Self {
            wallet_context,
            client,
            utxo_selector,
            fee_rate: FeeRate::from_sat_per_vb(10).unwrap(),
            change_address: sender,
            lock_time: None,
        })
    }

    /// Create a TransactionBuilder with pre-loaded UTXOs, avoiding redundant queries
    pub fn with_utxos(
        wallet_context: &'a WalletContext,
        client: Client,
        sender: Address,
        utxos: Vec<UTXOObjectView>,
    ) -> Self {
        let utxo_selector = UTXOSelector::with_utxos(client.clone(), sender.clone(), utxos);
        Self {
            wallet_context,
            client,
            utxo_selector,
            fee_rate: FeeRate::from_sat_per_vb(10).unwrap(),
            change_address: sender,
            lock_time: None,
        }
    }

    pub fn with_fee_rate(mut self, fee_rate: FeeRate) -> Self {
        self.fee_rate = fee_rate;
        self
    }

    pub fn with_lock_time(mut self, locktime: LockTime) -> Self {
        self.lock_time = Some(locktime);
        self
    }

    pub fn with_change_address(mut self, change_address: Address) -> Self {
        self.change_address = change_address;
        self
    }

    fn estimate_vbytes_with(inputs: usize, outputs: Vec<Address>) -> usize {
        Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: (0..inputs)
                .map(|_| TxIn {
                    previous_output: OutPoint::null(),
                    script_sig: ScriptBuf::new(),
                    sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                    witness: Witness::from_slice(&[&[0; Self::SCHNORR_SIGNATURE_SIZE]]),
                })
                .collect(),
            output: outputs
                .into_iter()
                .map(|address| TxOut {
                    value: Amount::from_sat(0),
                    script_pubkey: address.script_pubkey(),
                })
                .collect(),
        }
        .vsize()
    }

    pub async fn build_transfer(self, receipient: Address, amount: Amount) -> Result<Psbt> {
        self.build(vec![(receipient, amount)]).await
    }

    pub async fn build(mut self, outputs: Vec<(Address, Amount)>) -> Result<Psbt> {
        let total_output = outputs.iter().map(|(_, amount)| *amount).sum::<Amount>();
        let output_address = outputs
            .iter()
            .map(|(address, _)| address.clone())
            .collect::<Vec<_>>();
        let estimate_inputs = if self.utxo_selector.specific_utxos().is_empty() {
            1
        } else {
            self.utxo_selector.specific_utxos().len()
        };
        let estimate_fee = self
            .fee_rate
            .fee_vb(
                (Self::estimate_vbytes_with(estimate_inputs, output_address)
                    + Self::ADDITIONAL_INPUT_VBYTES
                    + Self::ADDITIONAL_OUTPUT_VBYTES) as u64,
            )
            .ok_or_else(|| anyhow!("Failed to estimate fee: {}", self.fee_rate))?;
        let mut utxos = self
            .utxo_selector
            .select_utxos(total_output + estimate_fee)
            .await?;
        let mut tx_inputs = vec![];
        let mut total_input = Amount::from_sat(0);
        for utxo in utxos.iter() {
            tx_inputs.push(Self::utxo_to_txin(utxo));
            total_input += utxo.amount();
        }

        let tx_outputs = outputs
            .into_iter()
            .map(|(address, amount)| TxOut {
                value: amount,
                script_pubkey: address.script_pubkey(),
            })
            .collect::<Vec<_>>();

        let mut tx = Transaction {
            version: Version::TWO,
            lock_time: self.lock_time.unwrap_or(LockTime::ZERO),
            input: tx_inputs,
            output: tx_outputs,
        };
        let fee = self
            .fee_rate
            .fee_vb(tx.vsize() as u64)
            .ok_or_else(|| anyhow!("Failed to estimate fee: {}", self.fee_rate))?;
        if fee > estimate_fee && total_input < total_output + fee {
            //we need to add more inputs
            let additional_utxos = self
                .utxo_selector
                .select_utxos(total_output + fee - total_input)
                .await?;
            tx.input
                .extend(additional_utxos.iter().map(Self::utxo_to_txin));
            total_input += additional_utxos
                .iter()
                .map(|utxo| utxo.amount())
                .sum::<Amount>();
            utxos.extend(additional_utxos);
        }

        let change = total_input - total_output - fee;
        if change > Amount::from_sat(0) {
            tx.output.push(TxOut {
                value: change,
                script_pubkey: self.change_address.script_pubkey(),
            });
        }
        let mut psbt = Psbt::from_unsigned_tx(tx)?;

        let multisign_account_module = self
            .client
            .as_module_binding::<multisign_account::MultisignAccountModule>();
        let mut multisign_psbt_cache: std::collections::HashMap<
            RoochAddress,
            PrecomputedMultisigPsbtData,
        > = std::collections::HashMap::new();
        let mut non_multisign_addresses: std::collections::HashSet<RoochAddress> =
            std::collections::HashSet::new();

        for (idx, utxo) in utxos.iter().enumerate() {
            let input = &mut psbt.inputs[idx];

            let bitcoin_addr = utxo.owner_bitcoin_address().ok_or_else(|| {
                anyhow!("Can not recognize the owner of UTXO {}", utxo.outpoint())
            })?;

            if bitcoin_addr.is_witness() {
                input.witness_utxo = Some(TxOut {
                    value: utxo.amount(),
                    script_pubkey: bitcoin_addr.script_pubkey()?,
                });
            } else {
                //TODO add non-witness utxo
                bail!("Non-witness UTXO is not supported yet");
            }

            let rooch_addr = bitcoin_addr.to_rooch_address();

            // Check if we already know this address is non-multisign
            if non_multisign_addresses.contains(&rooch_addr) {
                let kp = self.wallet_context.get_key_pair(&rooch_addr)?;
                input.bip32_derivation.insert(
                    kp.bitcoin_public_key()?.inner,
                    (Fingerprint::default(), Default::default()),
                );
                continue;
            }

            // Check if we have cached multisign data
            if let Some(cached) = multisign_psbt_cache.get(&rooch_addr) {
                debug!("Using cached precomputed PSBT data for {:?}", rooch_addr);
                multisign_account::update_psbt_with_precomputed_data(input, cached);
                continue;
            }

            // Need to check if this is a multisign account
            if multisign_account_module.is_multisign_account(rooch_addr.into())? {
                debug!(
                    "Fetching and precomputing multisign PSBT data for {:?}",
                    rooch_addr
                );
                let info = retry_rpc_call(|| async {
                    self.client
                        .rooch
                        .get_multisign_account_info(rooch_addr)
                        .await
                })
                .await?;
                debug!(
                    "Precomputing PSBT data for account with {} participants",
                    info.participants.data.len()
                );
                let precomputed = multisign_account::precompute_multisig_psbt_data(&info)?;
                multisign_psbt_cache.insert(rooch_addr, precomputed.clone());
                multisign_account::update_psbt_with_precomputed_data(input, &precomputed);
            } else {
                // Not a multisign account, remember this for future UTXOs
                non_multisign_addresses.insert(rooch_addr.clone());
                let kp = self.wallet_context.get_key_pair(&rooch_addr)?;
                input.bip32_derivation.insert(
                    kp.bitcoin_public_key()?.inner,
                    (Fingerprint::default(), Default::default()),
                );
            }
        }

        Ok(psbt)
    }

    fn utxo_to_txin(utxo: &UTXOObjectView) -> TxIn {
        TxIn {
            previous_output: utxo.outpoint().into(),
            script_sig: ScriptBuf::default(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::default(),
        }
    }
}
