// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    generator::{self, GeneratorLoader, InscribeSeed},
    operation::{AsSFT, DeployRecord, MergeRecord, MintRecord, Operation, SplitRecord},
    sft::{Content, SFT},
    GENERATOR_TICK,
};
use anyhow::{anyhow, bail, ensure, Result};
use bitcoin::{
    hashes::Hash,
    key::{TapTweak, TweakedKeypair},
    transaction::Version,
    BlockHash, OutPoint,
};
use bitcoincore_rpc::{
    bitcoincore_rpc_json::{ImportDescriptors, SignRawTransactionInput, Timestamp},
    json, RpcApi,
};
use clap::Parser;
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::bitcoin::{
    network::Network,
    ord::{Inscription, InscriptionID, InscriptionRecord, SatPoint},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use {
    bitcoin::{
        absolute::LockTime,
        address::NetworkUnchecked,
        blockdata::{opcodes, script},
        key::{Keypair, TweakedPublicKey},
        secp256k1::{
            self, constants::SCHNORR_SIGNATURE_SIZE, rand, All, Secp256k1, XOnlyPublicKey,
        },
        sighash::{Prevouts, SighashCache, TapSighashType},
        taproot::{
            ControlBlock, LeafVersion, Signature, TapLeafHash, TaprootBuilder, TaprootSpendInfo,
        },
        Address, Amount, FeeRate, PrivateKey, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid,
        Witness,
    },
    ciborium::Value,
};

const TARGET_POSTAGE: Amount = Amount::from_sat(10_000);

#[derive(Debug, Clone, Parser)]
pub struct InscribeOptions {
    #[arg(
        long,
        help = "Inscribe <SATPOINT>. This SatPoint will be used as mint seed."
    )]
    pub(crate) satpoint: Option<SatPoint>,
    #[arg(
        long,
        help = "Use <COMMIT_FEE_RATE> sats/vbyte for commit transaction.\nDefaults to <FEE_RATE> if unset."
    )]
    pub(crate) commit_fee_rate: Option<FeeRate>,
    #[arg(long, help = "Send inscription to <DESTINATION>.")]
    pub(crate) destination: Option<Address<NetworkUnchecked>>,
    #[arg(long, help = "Don't sign or broadcast transactions.")]
    pub(crate) dry_run: bool,
    #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB.")]
    pub(crate) fee_rate: FeeRate,
    #[arg(long, alias = "nobackup", help = "Do not back up recovery key.")]
    pub(crate) no_backup: bool,
    #[arg(
        long,
        alias = "nolimit",
        help = "Do not check that transactions are equal to or below the MAX_STANDARD_TX_WEIGHT of 400,000 weight units. Transactions over this limit are currently nonstandard and will not be relayed by bitcoind in its default configuration. Do not use this flag unless you understand the implications."
    )]
    pub(crate) no_limit: bool,
    #[arg(
        long,
        help = "Amount of postage to include in the inscription. Default `10000sat`."
    )]
    pub(crate) postage: Option<Amount>,
}

impl InscribeOptions {
    pub fn postage(&self) -> Amount {
        self.postage.unwrap_or(TARGET_POSTAGE)
    }

    pub fn commit_fee_rate(&self) -> FeeRate {
        self.commit_fee_rate.unwrap_or(self.fee_rate)
    }

    pub fn reveal_fee_rate(&self) -> FeeRate {
        self.fee_rate
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InscriptionOrId {
    Inscription(Inscription),
    Id(InscriptionID),
}

#[derive(Debug, Clone)]
pub struct InscribeContext {
    pub commit_tx: Transaction,
    pub reveal_tx: Transaction,
    pub signed_commit_tx_hex: Vec<u8>,
    pub signed_reveal_tx_hex: Vec<u8>,

    pub key_pairs: Vec<Keypair>,
    pub reveal_scripts: Vec<ScriptBuf>,
    pub control_blocks: Vec<ControlBlock>,
    pub taproot_spend_infos: Vec<TaprootSpendInfo>,
    pub commit_tx_addresses: Vec<Address>,

    pub utxos: BTreeMap<OutPoint, TxOut>,
    pub reveal_scripts_to_sign: Vec<ScriptBuf>,
    pub control_blocks_to_sign: Vec<ControlBlock>,
    pub commit_input_start_index: Option<usize>,

    pub total_burn_postage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InscribeOutput {
    commit_tx: Txid,
    reveal_tx: Txid,
    total_fees: u64,
    inscriptions: Vec<InscriptionOrId>,
}

pub struct Inscriber {
    context: WalletContext,
    option: InscribeOptions,
    inscriptions: Vec<InscriptionRecord>,
    inscriptions_to_burn: Vec<InscriptionID>,
    satpoint: SatPoint,
    destination: Address,
    network: bitcoin::Network,
    change_address: Address,
}

impl Inscriber {
    const SCHNORR_SIGNATURE_SIZE: usize = 64;

    pub async fn new(context: WalletContext, option: InscribeOptions) -> Result<Self> {
        let bitcoin_network = context.get_bitcoin_network().await?;
        let destination = match option.destination.clone() {
            Some(destination) => destination.require_network(bitcoin_network.into())?,
            None => {
                todo!()
            }
        };

        let satpoint = match option.satpoint.clone() {
            Some(satpoint) => {
                //TODO check the satpoint exists.
                satpoint
            }
            None => {
                //TODO
                //let utxo = context.select_utxo(&destination)?;
                SatPoint {
                    outpoint: OutPoint::null().into(),
                    offset: 0,
                }
            }
        };

        Ok(Self {
            context,
            option,
            inscriptions: Vec::new(),
            inscriptions_to_burn: Vec::new(),
            satpoint,
            destination,
            network: bitcoin_network.into(),
            change_address: todo!(),
        })
    }

    pub fn with_generator<P>(self, generator_name: String, generator_program: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let bytecode = std::fs::read(generator_program)?;
        let content = Content::new(generator::CONTENT_TYPE.to_string(), bytecode);
        let attributes = Value::Map(vec![(
            Value::Text("name".to_string()),
            Value::Text(generator_name.clone().into()),
        )]);

        let mint_record = MintRecord {
            sft: SFT {
                tick: GENERATOR_TICK.to_string(),
                amount: 1,
                attributes: Some(attributes),
                content: Some(content),
            },
        };

        Ok(self.with_operation(Operation::Mint(mint_record)))
    }

    pub fn with_deploy(
        self,
        tick: String,
        amount: u64,
        generator: Option<InscriptionID>,
        factory: Option<String>,
        repeat: u64,
        deploy_args: Vec<u8>,
    ) -> Result<Self> {
        match (&generator, &factory) {
            (Some(_), Some(_)) => bail!("generator and factory cannot be used together"),
            (None, None) => bail!("generator or factory must be provided"),
            _ => {}
        }
        //TODO check the generator exists.
        let deploy_record = DeployRecord {
            tick,
            amount,
            generator: generator.map(|generator| format!("/inscription/{}", generator)),
            factory,
            repeat,
            deploy_args,
        };
        Ok(self.with_operation(Operation::Deploy(deploy_record)))
    }

    pub fn with_mint(
        self,
        deploy_inscription: InscriptionID,
        user_input: Option<String>,
    ) -> Result<Self> {
        let operation = self.get_operation_by_inscription_id(deploy_inscription)?;
        let deploy_record = match operation {
            Operation::Deploy(deploy_record) => deploy_record,
            _ => bail!("deploy transaction must have a deploy operation"),
        };

        let generator_id = deploy_record.generator.as_ref().ok_or_else(|| {
            anyhow!("The deploy record does not have a generator inscription id, can not mint on Bitcoin")
        })?;
        let generator = GeneratorLoader::load(&self.context, generator_id)?;

        let seed_utxo = self.satpoint.outpoint.clone();
        let seed_tx = self.get_transaction(seed_utxo.txid())?;
        //TODO do not include the block_hash in the seed.
        let block_hash = BlockHash::all_zeros();
        let seed = InscribeSeed::new(block_hash, seed_utxo.into());

        let destination = self.destination.clone();

        let output = generator.inscribe_generate(
            &deploy_record.deploy_args,
            &seed,
            &destination,
            user_input,
        );

        let sft = SFT {
            tick: deploy_record.tick,
            amount: output.amount,
            attributes: output.attributes,
            content: output.content,
        };
        let mint_record = MintRecord { sft };

        Ok(self.with_operation(Operation::Mint(mint_record)))
    }

    pub fn with_split(
        self,
        asset_inscription_id: InscriptionID,
        amounts: Vec<u64>,
    ) -> Result<Self> {
        let operation = self.get_operation_by_inscription_id(asset_inscription_id)?;
        let sft = match operation {
            Operation::Mint(mint_record) => mint_record.as_sft(),
            Operation::Split(split_record) => split_record.as_sft(),
            Operation::Merge(merge_record) => merge_record.as_sft(),
            _ => bail!(
                "Inscription {} is not a valid SFT record",
                asset_inscription_id
            ),
        };

        ensure!(
            sft.amount >= amounts.iter().sum::<u64>(),
            "The total split amount exceeds the available SFT amount"
        );

        let mut remaining_amount = sft.amount;
        let mut result = self.with_burn(asset_inscription_id);

        let amounts_len = amounts.len();

        for (index, amount) in amounts.into_iter().enumerate() {
            let split_sft = SFT {
                tick: sft.tick.clone(),
                amount,
                attributes: sft.attributes.clone(),
                content: sft.content.clone(),
            };
            let split_record = SplitRecord { sft: split_sft };
            result = result.with_operation(Operation::Split(split_record));
            remaining_amount -= amount;

            if index == amounts_len - 1 {
                let remaining_sft = SFT {
                    tick: sft.tick.clone(),
                    amount: remaining_amount,
                    attributes: sft.attributes.clone(),
                    content: sft.content.clone(),
                };
                let split_record = SplitRecord { sft: remaining_sft };
                result = result.with_operation(Operation::Split(split_record));
            }
        }

        Ok(result)
    }

    pub fn with_merge(self, sft_inscription_ids: Vec<InscriptionID>) -> Result<Self> {
        ensure!(
            sft_inscription_ids.len() > 1,
            "At least two SFTs are required for merging"
        );

        let mut sft_to_merge = Vec::new();
        let mut result = self;

        for inscription_id in sft_inscription_ids {
            let operation = result.get_operation_by_inscription_id(inscription_id)?;
            let sft = match operation {
                Operation::Mint(mint_record) => mint_record.as_sft(),
                Operation::Split(split_record) => split_record.as_sft(),
                Operation::Merge(merge_record) => merge_record.as_sft(),
                _ => bail!("Inscription {} is not a minted SFT", inscription_id),
            };

            sft_to_merge.push(sft);
            result = result.with_burn(inscription_id);
        }

        let mut merged_sft = sft_to_merge[0].clone();
        for sft in sft_to_merge.iter().skip(1) {
            ensure!(
                merged_sft.tick == sft.tick,
                "All SFTs must have the same tick to be merged"
            );
            ensure!(
                merged_sft.attributes == sft.attributes,
                "All SFTs must have the same attributes to be merged"
            );
            ensure!(
                merged_sft.content == sft.content,
                "All SFTs must have the same content to be merged"
            );
            merged_sft.amount += sft.amount;
        }

        let merge_record = MergeRecord { sft: merged_sft };
        result = result.with_operation(Operation::Merge(merge_record));

        Ok(result)
    }

    pub fn with_burn(mut self, inscription_id: InscriptionID) -> Self {
        self.inscriptions_to_burn.push(inscription_id);
        self
    }

    fn with_operation(mut self, operation: Operation) -> Self {
        let inscription = operation.to_inscription();
        self.inscriptions.push(inscription);
        self
    }

    fn backup_recovery_key(context: &WalletContext, kp: TweakedKeypair) -> Result<()> {
        //TODO
        // let recovery_private_key = PrivateKey::new(
        //     recovery_key_pair.to_inner().secret_key(),
        //     wallet.chain().network(),
        // );

        // let bitcoin_client = wallet.bitcoin_client()?;

        // let info = bitcoin_client
        //     .get_descriptor_info(&format!("rawtr({})", recovery_private_key.to_wif()))?;

        // let response = bitcoin_client.import_descriptors(vec![ImportDescriptors {
        //     descriptor: format!("rawtr({})#{}", recovery_private_key.to_wif(), info.checksum),
        //     timestamp: Timestamp::Now,
        //     active: Some(false),
        //     range: None,
        //     next_index: None,
        //     internal: Some(false),
        //     label: Some("commit tx recovery key".to_string()),
        // }])?;

        // for result in response {
        //     if !result.success {
        //         return Err(anyhow!("commit tx recovery key import failed"));
        //     }
        // }

        Ok(())
    }

    fn calculate_fee(tx: &Transaction, utxos: &BTreeMap<OutPoint, TxOut>) -> u64 {
        tx.input
            .iter()
            .map(|txin| utxos.get(&txin.previous_output).unwrap().value.to_sat())
            .sum::<u64>()
            .checked_sub(
                tx.output
                    .iter()
                    .map(|txout| txout.value.to_sat())
                    .sum::<u64>(),
            )
            .unwrap()
    }

    fn create_reveal_script_and_control_block(
        inscription: &InscriptionRecord,
        secp256k1: &Secp256k1<All>,
    ) -> Result<(Keypair, ScriptBuf, ControlBlock, TaprootSpendInfo)> {
        let key_pair = Keypair::new(secp256k1, &mut rand::thread_rng());
        //UntweakedKeyPair::new(secp256k1, &mut rand::thread_rng());
        let (public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);

        //TODO
        let reveal_script = ScriptBuf::builder()
            .push_slice(public_key.serialize())
            .push_opcode(opcodes::all::OP_CHECKSIG)
            .into_script();

        // inscription
        //     .append_reveal_script_to_builder(
        //         ScriptBuf::builder()
        //             .push_slice(public_key.serialize())
        //             .push_opcode(opcodes::all::OP_CHECKSIG),
        //     )
        //     .into_script();

        let taproot_spend_info = TaprootBuilder::new()
            .add_leaf(0, reveal_script.clone())
            .expect("adding leaf should work")
            .finalize(secp256k1, public_key)
            .expect("finalizing taproot builder should work");

        let control_block = taproot_spend_info
            .control_block(&(reveal_script.clone(), LeafVersion::TapScript))
            .expect("should compute control block");

        Ok((key_pair, reveal_script, control_block, taproot_spend_info))
    }

    fn select_additional_inputs(
        &self,
        _ctx: &InscribeContext,
        additional_value: u64,
    ) -> Result<Vec<TxIn>> {
        // let mut selected_utxos = Vec::new();
        // let mut total_selected_value = 0;

        // let unspent_outputs = self.context.get_unspent_outputs()?;
        // let locked_outputs = self.context.get_locked_outputs()?;
        // let runic_outputs = self.context.get_runic_outputs()?;

        // for (outpoint, utxo) in unspent_outputs.iter() {
        //     if !locked_outputs.contains(outpoint) && !runic_outputs.contains(outpoint) {
        //         selected_utxos.push((*outpoint, utxo.clone()));
        //         total_selected_value += utxo.value;

        //         if total_selected_value >= additional_value {
        //             break;
        //         }
        //     }
        // }

        // ensure!(
        //     total_selected_value >= additional_value,
        //     "Insufficient funds in wallet to cover additional value"
        // );

        // let mut additional_inputs = Vec::new();
        // for (outpoint, _) in &selected_utxos {
        //     additional_inputs.push(TxIn {
        //         previous_output: *outpoint,
        //         script_sig: ScriptBuf::new(),
        //         sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        //         witness: Witness::new(),
        //     });
        // }

        // Ok(additional_inputs)

        todo!()
    }

    fn estimate_commit_tx_fee(&self, ctx: &InscribeContext) -> u64 {
        let input_count = ctx.commit_tx.input.len();
        let output_count = ctx.commit_tx.output.len();

        let mut estimated_commit_tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: (0..input_count)
                .map(|_| TxIn {
                    previous_output: OutPoint::null(),
                    script_sig: ScriptBuf::new(),
                    sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                    witness: Witness::from_slice(&[&[0; Self::SCHNORR_SIGNATURE_SIZE]]),
                })
                .collect(),
            output: (0..output_count)
                .map(|_| TxOut {
                    value: Amount::ZERO,
                    script_pubkey: ScriptBuf::new(),
                })
                .collect(),
        };

        for (index, output) in ctx.commit_tx.output.iter().enumerate() {
            estimated_commit_tx.output[index].script_pubkey = output.script_pubkey.clone();
        }

        self.option
            .commit_fee_rate()
            .fee_vb(estimated_commit_tx.vsize() as u64)
            .expect("fee should be valid")
            .to_sat()
    }

    fn estimate_reveal_tx_fee(
        &self,
        ctx: &InscribeContext,
        reveal_scripts: &Vec<ScriptBuf>,
        control_blocks: &Vec<ControlBlock>,
    ) -> u64 {
        let mut reveal_tx = ctx.reveal_tx.clone();
        let commit_input_start_index = ctx.commit_input_start_index.unwrap_or(0);

        for (current_index, txin) in reveal_tx.input.iter_mut().enumerate() {
            if current_index >= commit_input_start_index {
                let reveal_script = &reveal_scripts[current_index - commit_input_start_index];
                let control_block = &control_blocks[current_index - commit_input_start_index];

                txin.witness.push(
                    Signature::from_slice(&[0; SCHNORR_SIGNATURE_SIZE])
                        .unwrap()
                        .to_vec(),
                );
                txin.witness.push(reveal_script);
                txin.witness.push(&control_block.serialize());
            } else {
                // For inputs related to inscription destruction
                txin.witness.push(
                    Signature::from_slice(&[0; SCHNORR_SIGNATURE_SIZE])
                        .unwrap()
                        .to_vec(),
                );
                txin.witness.push(&[0; 33]); // Placeholder for public key
            }
        }

        self.option
            .reveal_fee_rate()
            .fee_vb(reveal_tx.vsize() as u64)
            .expect("fee should be valid")
            .to_sat()
    }

    fn assert_commit_transaction_balance(&self, ctx: &InscribeContext, msg: &str) {
        let tx = &ctx.commit_tx;
        let utxos = &ctx.utxos;

        let total_input: u64 = tx
            .input
            .iter()
            .map(|input| utxos.get(&input.previous_output).unwrap().value.to_sat())
            .sum();

        let total_output: u64 = tx.output.iter().map(|output| output.value.to_sat()).sum();

        let fee = self.estimate_commit_tx_fee(ctx);

        assert_eq!(total_input, total_output + fee, "{}", msg);
    }

    fn assert_reveal_transaction_balance(&self, ctx: &InscribeContext, msg: &str) {
        let tx = &ctx.reveal_tx;
        let utxos = &ctx.utxos;

        let total_input: u64 = tx
            .input
            .iter()
            .map(|input| utxos.get(&input.previous_output).unwrap().value.to_sat())
            .sum();

        let total_output: u64 = tx.output.iter().map(|output| output.value.to_sat()).sum();

        let fee = self.estimate_reveal_tx_fee(ctx, &ctx.reveal_scripts, &ctx.control_blocks);

        assert_eq!(total_input, total_output + fee, "{}", msg);
    }

    fn prepare_context(&self) -> Result<InscribeContext> {
        let commit_tx = Transaction {
            input: Vec::new(),
            output: Vec::new(),
            lock_time: LockTime::ZERO,
            version: Version::TWO,
        };

        let reveal_tx = Transaction {
            input: Vec::new(),
            output: Vec::new(),
            lock_time: LockTime::ZERO,
            version: Version::TWO,
        };

        let utxos = self.get_unspent_outputs()?;

        Ok(InscribeContext {
            commit_tx,
            reveal_tx,
            signed_commit_tx_hex: Vec::new(),
            signed_reveal_tx_hex: Vec::new(),

            key_pairs: Vec::new(),
            reveal_scripts: Vec::new(),
            control_blocks: Vec::new(),
            taproot_spend_infos: Vec::new(),
            commit_tx_addresses: Vec::new(),

            utxos,
            reveal_scripts_to_sign: Vec::new(),
            control_blocks_to_sign: Vec::new(),
            commit_input_start_index: None,
            total_burn_postage: None,
        })
    }

    fn build_commit(&self, ctx: &mut InscribeContext) -> Result<()> {
        let secp256k1 = Secp256k1::new();

        // set satpoint
        ctx.commit_tx.input.push(TxIn {
            previous_output: self.satpoint.outpoint.clone().into(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
        });

        let dust_threshold = self.destination.script_pubkey().dust_value();

        for inscription in &self.inscriptions {
            let (key_pair, reveal_script, control_block, taproot_spend_info) =
                Self::create_reveal_script_and_control_block(inscription, &secp256k1)?;

            let commit_tx_address =
                Address::p2tr_tweaked(taproot_spend_info.output_key(), self.network);

            let commit_tx_output = TxOut {
                script_pubkey: commit_tx_address.script_pubkey(),
                value: dust_threshold,
            };

            ctx.commit_tx.output.push(commit_tx_output);

            ctx.key_pairs.push(key_pair);
            ctx.reveal_scripts.push(reveal_script);
            ctx.control_blocks.push(control_block);
            ctx.taproot_spend_infos.push(taproot_spend_info);
            ctx.commit_tx_addresses.push(commit_tx_address);
        }

        Ok(())
    }

    fn build_revert(&self, ctx: &mut InscribeContext) -> Result<()> {
        // Process the logic of inscription destruction
        let mut total_burn_postage = 0;

        for inscription_to_burn in &self.inscriptions_to_burn {
            let inscription_id = inscription_to_burn.clone();
            let satpoint = self.get_inscription_satpoint_v2(inscription_id)?;
            let input = TxIn {
                previous_output: satpoint.outpoint(),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            };
            ctx.reveal_tx.input.push(input);

            let inscription_output = ctx
                .utxos
                .get(&satpoint.outpoint())
                .expect("inscription utxo not found");
            total_burn_postage += inscription_output.value.to_sat();
        }

        if self.inscriptions_to_burn.len() > 0 {
            let msg = b"bitseed".to_vec();
            let msg_push_bytes =
                script::PushBytesBuf::try_from(msg.clone()).expect("burn message should fit");

            let script = ScriptBuf::new_op_return(&msg_push_bytes);
            let output = TxOut {
                script_pubkey: script,
                value: Amount::from_sat(total_burn_postage),
            };
            ctx.reveal_tx.output.push(output);
            ctx.total_burn_postage = Some(total_burn_postage as f64);
        }

        // Process the logic of inscription revelation
        let commit_input_start_index = ctx.reveal_tx.input.len();

        for (index, ((_, control_block), reveal_script)) in ctx
            .commit_tx_addresses
            .iter()
            .zip(ctx.control_blocks.iter())
            .zip(ctx.reveal_scripts.iter())
            .enumerate()
        {
            // Add the commit transaction output as an input to the reveal transaction
            let commit_tx_outpoint = OutPoint {
                txid: ctx.commit_tx.compute_txid(),
                vout: index as u32,
            };
            let reveal_input = TxIn {
                previous_output: commit_tx_outpoint,
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            };
            ctx.reveal_tx.input.push(reveal_input);

            // Add the inscription output to the reveal transaction
            let reveal_output = TxOut {
                script_pubkey: self.destination.script_pubkey(),
                value: self.option.postage(),
            };
            ctx.reveal_tx.output.push(reveal_output);

            // Save the reveal script and control block for signing later
            ctx.reveal_scripts_to_sign.push(reveal_script.clone());
            ctx.control_blocks_to_sign.push(control_block.clone());
        }

        // Set the commit input index in the context
        ctx.commit_input_start_index = Some(commit_input_start_index);

        Ok(())
    }

    fn update_fees(&self, ctx: &mut InscribeContext) -> Result<()> {
        let dust_threshold = self.destination.script_pubkey().dust_value().to_sat();

        let actual_reveal_fee =
            self.estimate_reveal_tx_fee(ctx, &ctx.reveal_scripts, &ctx.control_blocks);
        let total_new_postage = self.option.postage().to_sat() * self.inscriptions.len() as u64;

        let reveal_additional_fee = actual_reveal_fee + total_new_postage;

        if reveal_additional_fee > 0 {
            let mut remaining_fee = reveal_additional_fee;

            for output in ctx.commit_tx.output.iter_mut() {
                remaining_fee -= output.value.to_sat();
            }

            // If there's still remaining fee, add it to the last output
            if remaining_fee > 0 {
                let last_output = ctx
                    .commit_tx
                    .output
                    .last_mut()
                    .expect("there should be at least one output");
                last_output.value += Amount::from_sat(remaining_fee);
            }
        }

        // Check if recharge is required
        let mut commit_fee: i64 = self.estimate_commit_tx_fee(ctx) as i64;
        let mut change_value = Self::calculate_fee(&ctx.commit_tx, &ctx.utxos) as i64 - commit_fee;

        if change_value < 0 {
            let additional_inputs =
                self.select_additional_inputs(ctx, (0 - change_value) as u64)?;
            for input in additional_inputs {
                ctx.commit_tx.input.push(input);
            }
        }

        // Check if change is needed
        commit_fee = self.estimate_commit_tx_fee(ctx) as i64;
        change_value = Self::calculate_fee(&ctx.commit_tx, &ctx.utxos) as i64 - commit_fee;

        if change_value > dust_threshold as i64 {
            ctx.commit_tx.output.push(TxOut {
                script_pubkey: self.change_address.script_pubkey(),
                value: Amount::from_sat(change_value as u64),
            });

            // Recalculate the actual commit fee, considering the impact of the change output
            let new_commit_fee = self.estimate_commit_tx_fee(ctx) as i64;

            // Adjust the change amount to compensate for the fee change
            let fee_difference = new_commit_fee - commit_fee;
            change_value -= fee_difference as i64;

            if change_value <= dust_threshold as i64 {
                // If the adjusted change amount is less than or equal to the dust threshold, remove the change output
                ctx.commit_tx.output.pop();
            } else {
                // Update the amount of the change output
                ctx.commit_tx.output.last_mut().unwrap().value =
                    Amount::from_sat(change_value as u64);
            }
        }

        // Update the reveal transaction inputs to reference the new commit transaction outputs
        let new_commit_txid = ctx.commit_tx.compute_txid();
        for (index, input) in ctx.reveal_tx.input.iter_mut().enumerate() {
            if index >= ctx.commit_input_start_index.unwrap() {
                input.previous_output.txid = new_commit_txid;

                ctx.utxos.insert(
                    input.previous_output,
                    ctx.commit_tx.output[input.previous_output.vout as usize].clone(),
                );
            }
        }

        // Check commit fee
        self.assert_commit_transaction_balance(
            ctx,
            "commit transaction input, output, and fee do not match",
        );
        self.assert_reveal_transaction_balance(
            ctx,
            "reveal transaction input, output, and fee do not match",
        );

        Ok(())
    }

    fn sign(&self, ctx: &mut InscribeContext) -> Result<()> {
        //let bitcoin_client = self.context.bitcoin_client()?;

        // Sign the commit transaction
        //TODO
        // ctx.signed_commit_tx_hex = bitcoin_client
        //     .sign_raw_transaction_with_wallet(&ctx.commit_tx, None, None)?
        //     .hex;

        // Sign the inputs for inscription revelation
        let commit_input_start_index = ctx.commit_input_start_index.unwrap();

        let prevouts: Vec<_> = ctx
            .reveal_tx
            .input
            .iter()
            .map(|tx_in| {
                ctx.utxos
                    .get(&tx_in.previous_output)
                    .clone()
                    .expect("prevout not found")
            })
            .collect();

        let mut sighash_cache = SighashCache::new(&mut ctx.reveal_tx);
        for (index, ((reveal_script, control_block), keypair)) in ctx
            .reveal_scripts_to_sign
            .iter()
            .zip(ctx.control_blocks_to_sign.iter())
            .zip(ctx.key_pairs.iter())
            .enumerate()
        {
            let sighash = sighash_cache
                .taproot_script_spend_signature_hash(
                    commit_input_start_index + index,
                    &Prevouts::All(&prevouts),
                    TapLeafHash::from_script(reveal_script, LeafVersion::TapScript),
                    TapSighashType::Default,
                )
                .expect("failed to compute sighash");

            let secp = Secp256k1::new();
            let message =
                secp256k1::Message::from_slice(sighash.as_ref()).expect("failed to create message");
            let sig = secp.sign_schnorr(&message, &keypair);

            let witness = sighash_cache
                .witness_mut(commit_input_start_index + index)
                .expect("getting mutable witness reference should work");

            witness.push(
                Signature {
                    signature: sig,
                    sighash_type: TapSighashType::Default,
                }
                .to_vec(),
            );

            witness.push(reveal_script);
            witness.push(&control_block.serialize());
        }

        // Sign the reveal transaction (inscription destruction part)
        let inscription_destroy_prevouts: Vec<SignRawTransactionInput> = ctx
            .reveal_tx
            .input
            .iter()
            .map(|input| {
                let prevout = input.previous_output;
                let utxo = ctx.utxos.get(&prevout).expect("utxo not found").clone();
                SignRawTransactionInput {
                    txid: prevout.txid,
                    vout: prevout.vout,
                    script_pub_key: utxo.script_pubkey,
                    redeem_script: None,
                    amount: Some(utxo.value),
                }
            })
            .collect();

        // ctx.signed_reveal_tx_hex = bitcoin_client
        //     .sign_raw_transaction_with_wallet(
        //         &ctx.reveal_tx,
        //         Some(&inscription_destroy_prevouts),
        //         Some(json::SigHashType::from(
        //             bitcoin::sighash::EcdsaSighashType::All,
        //         )),
        //     )?
        //     .hex;

        Ok(())
    }

    fn backup_keys(&self, ctx: &mut InscribeContext) -> Result<()> {
        if self.option.no_backup {
            return Ok(());
        }

        let secp256k1 = Secp256k1::new();

        for ((key_pair, commit_tx_address), taproot_spend_info) in ctx
            .key_pairs
            .iter()
            .zip(ctx.commit_tx_addresses.iter())
            .zip(ctx.taproot_spend_infos.iter())
        {
            let recovery_key_pair =
                key_pair.tap_tweak(&secp256k1, taproot_spend_info.merkle_root());
            let (x_only_pub_key, _parity) = recovery_key_pair.to_inner().x_only_public_key();
            assert_eq!(
                Address::p2tr_tweaked(
                    TweakedPublicKey::dangerous_assume_tweaked(x_only_pub_key),
                    self.network,
                ),
                commit_tx_address.clone(),
                "commit_tx_address invalid"
            );

            Self::backup_recovery_key(&self.context, recovery_key_pair)?;
        }

        Ok(())
    }

    fn boardcaset_tx(&self, ctx: &mut InscribeContext) -> Result<InscribeOutput> {
        let total_fees = Self::calculate_fee(&ctx.commit_tx, &ctx.utxos)
            + Self::calculate_fee(&ctx.reveal_tx, &ctx.utxos);

        let origin_reveal_txid = ctx.reveal_tx.compute_txid();
        let reveal_input_count = ctx.reveal_scripts_to_sign.len();
        let inscriptions: Vec<_> = (0..reveal_input_count)
            .map(|index| InscriptionID::new(origin_reveal_txid.clone(), index as u32))
            .map(InscriptionOrId::Id)
            .collect();

        if self.option.dry_run {
            let origin_commit_txid = ctx.commit_tx.compute_txid();

            return Ok(InscribeOutput {
                commit_tx: origin_commit_txid,
                reveal_tx: origin_reveal_txid,
                total_fees,
                inscriptions,
            });
        }

        let commit_txid = match self.send_raw_transaction(&ctx.signed_commit_tx_hex, None, None) {
            Ok(txid) => txid,
            Err(err) => return Err(anyhow!("Failed to send commit transaction: {err}")),
        };

        let reveal_txid = match self.send_raw_transaction(&ctx.signed_reveal_tx_hex, None, ctx.total_burn_postage) {
            Ok(txid) => txid,
            Err(err) => {
                return Err(anyhow!(
                "Failed to send reveal transaction: {err}\nCommit tx {commit_txid} will be recovered once mined"
                ))
            }
        };

        assert_eq!(
            origin_reveal_txid, reveal_txid,
            "reveal txid should be equal"
        );

        Ok(InscribeOutput {
            commit_tx: commit_txid,
            reveal_tx: reveal_txid,
            total_fees,
            inscriptions,
        })
    }

    pub fn inscribe(&self) -> Result<InscribeOutput> {
        let mut ctx = self.prepare_context()?;

        self.build_commit(&mut ctx)?;
        self.build_revert(&mut ctx)?;
        self.update_fees(&mut ctx)?;
        self.sign(&mut ctx)?;
        self.backup_keys(&mut ctx)?;
        self.boardcaset_tx(&mut ctx)
    }

    pub fn get_operation_by_inscription_id(
        &self,
        inscription_id: InscriptionID,
    ) -> Result<Operation> {
        // let tx = self.get_raw_transaction(&inscription_id.txid)?;
        // let inscriptions = ParsedEnvelope::from_transaction(&tx);

        // let envelope = inscriptions
        //     .into_iter()
        //     .nth(inscription_id.index as usize)
        //     .ok_or_else(|| anyhow!("Inscription not found in the transaction"))?;

        // Operation::from_inscription(envelope.payload)

        todo!()
    }

    pub fn get_transaction(&self, txid: Txid) -> Result<Transaction> {
        todo!()
    }

    pub fn send_raw_transaction(
        &self,
        tx_hex: &Vec<u8>,
        maxfeerate: Option<f64>,
        maxburnamount: Option<f64>,
    ) -> Result<bitcoin::Txid> {
        // let bitcoin_client = self.bitcoin_client()?;

        // // Prepare the parameters for the RPC call
        // let mut params = vec![tx.raw_hex().into()];

        // // Add maxfeerate and maxburnamount to the params if they are Some
        // if let Some(feerate) = maxfeerate {
        //     params.push(serde_json::to_value(feerate).unwrap());
        // } else {
        //     params.push(serde_json::to_value(0.10).unwrap());
        // }

        // if let Some(burnamount) = maxburnamount {
        //     params.push(serde_json::to_value(burnamount).unwrap());
        // } else {
        //     params.push(serde_json::to_value(0.0).unwrap());
        // }

        // // Make the RPC call
        // let tx_id: bitcoin::Txid = bitcoin_client.call("sendrawtransaction", &params)?;

        //Ok(tx_id)
        todo!()
    }

    fn get_unspent_outputs(&self) -> Result<BTreeMap<OutPoint, TxOut>> {
        todo!()
    }

    fn get_inscription_satpoint_v2(&self, inscription_id: InscriptionID) -> Result<SatPoint> {
        todo!()
    }
}
