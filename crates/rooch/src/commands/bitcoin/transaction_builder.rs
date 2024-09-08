// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::utxo_selector::UTXOSelector;
use anyhow::{anyhow, bail, Result};
use bitcoin::{
    absolute::LockTime, transaction::Version, Address, Amount, FeeRate, OutPoint, Psbt, ScriptBuf,
    Sequence, Transaction, TxIn, TxOut, Witness,
};
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_client::Client;
use rooch_types::bitcoin::utxo::UTXO;

#[derive(Debug)]
pub struct TransactionBuilder {
    utxo_selector: UTXOSelector,
    fee_rate: FeeRate,
    change_address: Address,
    lock_time: Option<LockTime>,
}

impl TransactionBuilder {
    const ADDITIONAL_INPUT_VBYTES: usize = 58;
    const ADDITIONAL_OUTPUT_VBYTES: usize = 43;
    const SCHNORR_SIGNATURE_SIZE: usize = 64;

    pub async fn new(
        client: Client,
        sender: Address,
        inputs: Vec<ObjectID>,
        skip_seal_check: bool,
    ) -> Result<Self> {
        let utxo_selector =
            UTXOSelector::new(client, sender.clone(), inputs, skip_seal_check).await?;
        Ok(Self {
            utxo_selector,
            fee_rate: FeeRate::from_sat_per_vb(10).unwrap(),
            change_address: sender,
            lock_time: None,
        })
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
        let utxos = self.select_utxos(total_output + estimate_fee).await?;
        let mut tx_inputs = vec![];
        let mut total_input = Amount::from_sat(0);
        for utxo in utxos {
            tx_inputs.push(Self::utxo_to_txin(&utxo));
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
            let utxos = self.select_utxos(total_output + fee - total_input).await?;
            tx.input.extend(utxos.iter().map(Self::utxo_to_txin));
            total_input += utxos.iter().map(|utxo| utxo.amount()).sum::<Amount>();
        }

        let change = total_input - total_output - fee;
        if change > Amount::from_sat(0) {
            tx.output.push(TxOut {
                value: change,
                script_pubkey: self.change_address.script_pubkey(),
            });
        }
        let psbt = Psbt::from_unsigned_tx(tx)?;
        Ok(psbt)
    }

    async fn select_utxos(&mut self, expected_amount: Amount) -> Result<Vec<UTXO>> {
        let mut utxos = vec![];
        let mut total_input = Amount::from_sat(0);
        while total_input < expected_amount {
            let utxo = self.utxo_selector.next_utxo().await?;
            if utxo.is_none() {
                bail!("not enough BTC funds");
            }
            let utxo = utxo.unwrap();
            total_input += utxo.amount();
            utxos.push(utxo);
        }
        Ok(utxos)
    }

    fn utxo_to_txin(utxo: &UTXO) -> TxIn {
        TxIn {
            previous_output: utxo.outpoint().into(),
            script_sig: ScriptBuf::default(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::default(),
        }
    }
}
