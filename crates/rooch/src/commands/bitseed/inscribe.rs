// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    generator::{self, Generator, InscribeSeed},
    operation::{AsSFT, DeployRecord, MergeRecord, MintRecord, Operation, SplitRecord},
    sft::{Content, SFT},
    GENERATOR_TICK,
};
use crate::commands::{
    bitcoin::utxo_selector::UTXOSelector,
    bitseed::generator::{wasm::wasm_generator::WASMGenerator, CONTENT_TYPE},
};
use anyhow::{anyhow, bail, ensure, Result};
use bitcoin::{
    key::{TapTweak, TweakedKeypair},
    script::PushBytesBuf,
    secp256k1::Message,
    transaction::Version,
    EcdsaSighashType, OutPoint,
};
use clap::Parser;
use rooch_rpc_api::jsonrpc_types::btc::{ord::InscriptionObjectView, utxo::UTXOObjectView};
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::{
    address::{BitcoinAddress, ParsedAddress},
    bitcoin::ord::{Inscription, InscriptionID, InscriptionRecord, SatPoint},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path, str::FromStr};
use tracing::debug;
use {
    bitcoin::{
        absolute::LockTime,
        blockdata::{opcodes, script},
        key::{Keypair, TweakedPublicKey},
        secp256k1::{constants::SCHNORR_SIGNATURE_SIZE, rand, All, Secp256k1, XOnlyPublicKey},
        sighash::{Prevouts, SighashCache, TapSighashType},
        taproot::{
            ControlBlock, LeafVersion, Signature, TapLeafHash, TaprootBuilder, TaprootSpendInfo,
        },
        Address, Amount, FeeRate, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
    },
    ciborium::Value,
};

const TARGET_POSTAGE: Amount = Amount::from_sat(10_000);

#[derive(Debug, Clone, Parser)]
pub struct InscribeOptions {
    /// The sender address of the transaction, if not specified, the active address will be used
    #[clap(long, short = 's', default_value = "default")]
    pub(crate) sender: ParsedAddress,

    /// Send inscription to <DESTINATION>, if not specified, the sender address will be used
    #[arg(long)]
    pub(crate) destination: Option<ParsedAddress>,

    /// Use <CHANGE_ADDRESS> as the change address, if not specified, the sender address will be used
    #[arg(long)]
    pub(crate) change_address: Option<ParsedAddress>,

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
    Inscription(Box<Inscription>),
    Id(InscriptionID),
}

#[derive(Debug, Clone)]
pub struct InscribeContext {
    pub commit_tx: Transaction,
    pub reveal_tx: Transaction,
    pub signed_commit_tx: Option<Transaction>,
    pub signed_reveal_tx: Option<Transaction>,

    pub key_pairs: Vec<Keypair>,
    pub reveal_scripts: Vec<ScriptBuf>,
    pub control_blocks: Vec<ControlBlock>,
    pub taproot_spend_infos: Vec<TaprootSpendInfo>,
    pub commit_tx_addresses: Vec<Address>,

    pub utxos: BTreeMap<OutPoint, TxOut>,
    pub reveal_scripts_to_sign: Vec<ScriptBuf>,
    pub control_blocks_to_sign: Vec<ControlBlock>,
    pub reveal_input_start_index: Option<usize>,

    pub total_burn_postage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InscribeOutput {
    commit_tx: Txid,
    reveal_tx: Txid,
    total_fees: Amount,
    inscriptions: Vec<InscriptionOrId>,
}

pub struct Inscriber {
    context: WalletContext,
    utxo_selector: UTXOSelector,
    option: InscribeOptions,
    inscriptions: Vec<InscriptionRecord>,
    inscriptions_to_burn: Vec<InscriptionID>,
    satpoint: (SatPoint, UTXOObjectView),
    network: bitcoin::Network,
    destination: Address,
    change_address: Address,
}

impl Inscriber {
    const SCHNORR_SIGNATURE_SIZE: usize = 64;

    pub async fn new(context: WalletContext, option: InscribeOptions) -> Result<Self> {
        let bitcoin_network = context.get_bitcoin_network().await?;
        let sender = context
            .resolve_bitcoin_address(option.sender.clone())
            .await?;
        let destination = match option.destination.clone() {
            Some(destination) => context.resolve_bitcoin_address(destination).await?,
            None => sender.clone(),
        };
        let change_address = match option.change_address.clone() {
            Some(change_address) => context.resolve_bitcoin_address(change_address).await?,
            None => sender.clone(),
        };

        let mut utxo_selector = UTXOSelector::new(
            context.get_client().await?,
            sender.to_bitcoin_address(bitcoin_network)?,
            Vec::new(),
            false,
        )
        .await?;

        let (satpoint, utxo) = match option.satpoint.clone() {
            Some(satpoint) => {
                let utxo_view = utxo_selector.get_utxo(&satpoint.outpoint).await?;
                (satpoint, utxo_view)
            }
            None => {
                let utxo_view = utxo_selector
                    .next_utxo()
                    .await?
                    .ok_or_else(|| anyhow!("No UTXO found"))?;
                let outpoint = utxo_view.outpoint();
                (
                    SatPoint {
                        outpoint,
                        offset: 0,
                    },
                    utxo_view,
                )
            }
        };

        Ok(Self {
            context,
            utxo_selector,
            option,
            inscriptions: Vec::new(),
            inscriptions_to_burn: Vec::new(),
            satpoint: (satpoint, utxo),
            network: bitcoin_network.into(),
            destination: destination.to_bitcoin_address(bitcoin_network)?,
            change_address: change_address.to_bitcoin_address(bitcoin_network)?,
        })
    }

    pub async fn with_generator<P>(
        self,
        generator_name: String,
        generator_program: P,
    ) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let bytecode = std::fs::read(generator_program)?;
        let content = Content::new(generator::CONTENT_TYPE.to_string(), bytecode);
        let attributes = Value::Map(vec![(
            Value::Text("name".to_string()),
            Value::Text(generator_name.clone()),
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

    pub async fn with_deploy(
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

    pub async fn with_mint(
        self,
        deploy_inscription: InscriptionID,
        user_input: Option<String>,
    ) -> Result<Self> {
        let operation = self
            .get_operation_by_inscription_id(deploy_inscription)
            .await?;
        let deploy_record = match operation {
            Operation::Deploy(deploy_record) => deploy_record,
            _ => bail!("deploy transaction must have a deploy operation"),
        };

        let generator_id = deploy_record.generator.as_ref().ok_or_else(|| {
            anyhow!("The deploy record does not have a generator inscription id, can not mint on Bitcoin")
        })?;
        let generator = self.load_generator(generator_id).await?;

        let seed_utxo = self.satpoint.0.outpoint.clone();

        let seed = InscribeSeed::new(seed_utxo.into());

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

    pub async fn with_split(
        self,
        asset_inscription_id: InscriptionID,
        amounts: Vec<u64>,
    ) -> Result<Self> {
        let operation = self
            .get_operation_by_inscription_id(asset_inscription_id)
            .await?;
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
        let mut result = self.with_burn(asset_inscription_id).await;

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

    pub async fn with_merge(self, sft_inscription_ids: Vec<InscriptionID>) -> Result<Self> {
        ensure!(
            sft_inscription_ids.len() > 1,
            "At least two SFTs are required for merging"
        );

        let mut sft_to_merge = Vec::new();
        let mut result = self;

        for inscription_id in sft_inscription_ids {
            let operation = result
                .get_operation_by_inscription_id(inscription_id)
                .await?;
            let sft = match operation {
                Operation::Mint(mint_record) => mint_record.as_sft(),
                Operation::Split(split_record) => split_record.as_sft(),
                Operation::Merge(merge_record) => merge_record.as_sft(),
                _ => bail!("Inscription {} is not a minted SFT", inscription_id),
            };

            sft_to_merge.push(sft);
            result = result.with_burn(inscription_id).await;
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

    pub async fn with_burn(mut self, inscription_id: InscriptionID) -> Self {
        self.inscriptions_to_burn.push(inscription_id);
        self
    }

    fn with_operation(mut self, operation: Operation) -> Self {
        let inscription = operation.to_inscription();
        self.inscriptions.push(inscription);
        self
    }

    fn backup_recovery_key(_context: &WalletContext, kp: TweakedKeypair) -> Result<()> {
        //TODO back up recovery key

        println!(
            "Recovery key: {}",
            hex::encode(kp.to_inner().secret_bytes())
        );

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

    fn calculate_fee(tx: &Transaction, utxos: &BTreeMap<OutPoint, TxOut>) -> Amount {
        tx.input
            .iter()
            .map(|txin| utxos.get(&txin.previous_output).unwrap().value)
            .sum::<Amount>()
            .checked_sub(tx.output.iter().map(|txout| txout.value).sum::<Amount>())
            .unwrap()
    }

    fn create_reveal_script_and_control_block(
        inscription: &InscriptionRecord,
        secp256k1: &Secp256k1<All>,
    ) -> Result<(Keypair, ScriptBuf, ControlBlock, TaprootSpendInfo)> {
        let key_pair = Keypair::new(secp256k1, &mut rand::thread_rng());
        let (public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);

        let reveal_script = inscription
            .append_reveal_script_to_builder(
                ScriptBuf::builder()
                    .push_slice(public_key.serialize())
                    .push_opcode(opcodes::all::OP_CHECKSIG),
            )
            .into_script();

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

    async fn select_additional_inputs(
        &mut self,
        ctx: &mut InscribeContext,
        additional_value: Amount,
    ) -> Result<Vec<TxIn>> {
        let utxos = self.utxo_selector.select_utxos(additional_value).await?;

        let mut additional_inputs = Vec::new();

        for utxo in utxos {
            let input = TxIn {
                previous_output: utxo.outpoint().into(),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            };
            ctx.utxos.insert(utxo.outpoint().into(), utxo.tx_output()?);
            additional_inputs.push(input);
        }

        Ok(additional_inputs)
    }

    fn estimate_commit_tx_fee(&self, ctx: &InscribeContext) -> Amount {
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
    }

    fn estimate_reveal_tx_fee(
        &self,
        ctx: &InscribeContext,
        reveal_scripts: &[ScriptBuf],
        control_blocks: &[ControlBlock],
    ) -> Amount {
        let mut reveal_tx = ctx.reveal_tx.clone();
        let reveal_input_start_index = ctx.reveal_input_start_index.unwrap_or(0);

        for (current_index, txin) in reveal_tx.input.iter_mut().enumerate() {
            if current_index >= reveal_input_start_index {
                let reveal_script = &reveal_scripts[current_index - reveal_input_start_index];
                let control_block = &control_blocks[current_index - reveal_input_start_index];

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
                txin.witness.push([0; 33]); // Placeholder for public key
            }
        }

        self.option
            .reveal_fee_rate()
            .fee_vb(reveal_tx.vsize() as u64)
            .expect("fee should be valid")
    }

    fn assert_commit_transaction_balance(&self, ctx: &InscribeContext, msg: &str) {
        let tx = &ctx.commit_tx;
        let utxos = &ctx.utxos;

        let total_input: Amount = tx
            .input
            .iter()
            .map(|input| utxos.get(&input.previous_output).unwrap().value)
            .sum();

        let total_output: Amount = tx.output.iter().map(|output| output.value).sum();

        let fee = self.estimate_commit_tx_fee(ctx);

        assert_eq!(total_input, total_output + fee, "{}", msg);
    }

    fn assert_reveal_transaction_balance(&self, ctx: &InscribeContext, msg: &str) {
        let tx = &ctx.reveal_tx;
        let utxos = &ctx.utxos;

        let total_input: Amount = tx
            .input
            .iter()
            .map(|input| utxos.get(&input.previous_output).unwrap().value)
            .sum();

        let total_output: Amount = tx.output.iter().map(|output| output.value).sum();

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

        let mut utxos = BTreeMap::new();
        utxos.insert(
            self.satpoint.0.outpoint.clone().into(),
            self.satpoint.1.tx_output()?,
        );

        Ok(InscribeContext {
            commit_tx,
            reveal_tx,
            signed_commit_tx: None,
            signed_reveal_tx: None,

            key_pairs: Vec::new(),
            reveal_scripts: Vec::new(),
            control_blocks: Vec::new(),
            taproot_spend_infos: Vec::new(),
            commit_tx_addresses: Vec::new(),

            utxos,
            reveal_scripts_to_sign: Vec::new(),
            control_blocks_to_sign: Vec::new(),
            reveal_input_start_index: None,
            total_burn_postage: None,
        })
    }

    fn build_commit(&self, ctx: &mut InscribeContext) -> Result<()> {
        let secp256k1 = Secp256k1::new();

        // set satpoint
        ctx.commit_tx.input.push(TxIn {
            previous_output: self.satpoint.0.outpoint.clone().into(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
        });

        let dust_threshold = self.destination.script_pubkey().minimal_non_dust();

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

    async fn build_reveal(&self, ctx: &mut InscribeContext) -> Result<()> {
        // Process the logic of inscription destruction
        let mut total_burn_postage = Amount::ZERO;

        for inscription_to_burn in &self.inscriptions_to_burn {
            let inscription_id = *inscription_to_burn;
            let satpoint = self.get_inscription_satpoint(inscription_id).await?;
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
            total_burn_postage += inscription_output.value;
        }

        if !self.inscriptions_to_burn.is_empty() {
            let msg = b"bitseed".to_vec();
            let msg_push_bytes =
                script::PushBytesBuf::try_from(msg.clone()).expect("burn message should fit");

            let script = ScriptBuf::new_op_return(msg_push_bytes);
            let output = TxOut {
                script_pubkey: script,
                value: total_burn_postage,
            };
            ctx.reveal_tx.output.push(output);
            ctx.total_burn_postage = Some(total_burn_postage.to_btc());
        }

        // Process the logic of inscription revelation
        let reveal_input_start_index = ctx.reveal_tx.input.len();

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
        ctx.reveal_input_start_index = Some(reveal_input_start_index);

        Ok(())
    }

    async fn update_fees(&mut self, ctx: &mut InscribeContext) -> Result<()> {
        let dust_threshold = self.destination.script_pubkey().minimal_non_dust().to_sat();

        let actual_reveal_fee =
            self.estimate_reveal_tx_fee(ctx, &ctx.reveal_scripts, &ctx.control_blocks);
        let total_new_postage =
            Amount::from_sat(self.option.postage().to_sat() * self.inscriptions.len() as u64);

        let reveal_additional_fee = actual_reveal_fee + total_new_postage;

        if reveal_additional_fee > Amount::ZERO {
            let mut remaining_fee = reveal_additional_fee;

            for output in ctx.commit_tx.output.iter_mut() {
                remaining_fee -= output.value;
            }

            // If there's still remaining fee, add it to the last output
            if remaining_fee > Amount::ZERO {
                let last_output = ctx
                    .commit_tx
                    .output
                    .last_mut()
                    .expect("there should be at least one output");
                last_output.value += remaining_fee;
            }
        }

        // Check if recharge is required
        let mut commit_fee: i64 = self.estimate_commit_tx_fee(ctx).to_sat() as i64;
        let mut change_value =
            Self::calculate_fee(&ctx.commit_tx, &ctx.utxos).to_sat() as i64 - commit_fee;

        if change_value < 0 {
            let additional_inputs = self
                .select_additional_inputs(ctx, Amount::from_sat((0 - change_value) as u64))
                .await?;
            for input in additional_inputs {
                ctx.commit_tx.input.push(input);
            }
        }

        // Check if change is needed
        commit_fee = self.estimate_commit_tx_fee(ctx).to_sat() as i64;
        change_value = Self::calculate_fee(&ctx.commit_tx, &ctx.utxos).to_sat() as i64 - commit_fee;

        if change_value > dust_threshold as i64 {
            ctx.commit_tx.output.push(TxOut {
                script_pubkey: self.change_address.script_pubkey(),
                value: Amount::from_sat(change_value as u64),
            });

            // Recalculate the actual commit fee, considering the impact of the change output
            let new_commit_fee = self.estimate_commit_tx_fee(ctx).to_sat() as i64;

            // Adjust the change amount to compensate for the fee change
            let fee_difference = new_commit_fee - commit_fee;
            change_value -= fee_difference;

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
            if index >= ctx.reveal_input_start_index.unwrap() {
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

    fn sign_commit_tx(&self, ctx: &mut InscribeContext) -> Result<()> {
        let commit_input_utxos = ctx
            .commit_tx
            .input
            .iter()
            .map(|tx_in| {
                ctx.utxos
                    .get(&tx_in.previous_output)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("utxo {} not found", tx_in.previous_output))
            })
            .collect::<Result<Vec<_>>>()?;

        debug!("Commit tx before sign: {:?}", ctx.commit_tx);
        ctx.signed_commit_tx = Some(sign_tx(&self.context, &ctx.commit_tx, &commit_input_utxos)?);
        debug!("Commit tx after sign: {:?}", ctx.signed_commit_tx);
        Ok(())
    }

    fn sign_reveal_tx(&self, ctx: &mut InscribeContext) -> Result<()> {
        let reveal_input_start_index = ctx.reveal_input_start_index.unwrap();

        let prevouts: Vec<_> = ctx
            .reveal_tx
            .input
            .iter()
            .map(|tx_in| {
                ctx.utxos
                    .get(&tx_in.previous_output)
                    .cloned()
                    .expect("prevout not found")
            })
            .collect();

        let mut signed_reveal_tx = ctx.reveal_tx.clone();

        // sign the reveal inscription input
        debug!("Reveal tx before sign: {:?}", signed_reveal_tx);

        let mut sighash_cache = SighashCache::new(&mut signed_reveal_tx);
        for (index, ((reveal_script, control_block), keypair)) in ctx
            .reveal_scripts_to_sign
            .iter()
            .zip(ctx.control_blocks_to_sign.iter())
            .zip(ctx.key_pairs.iter())
            .enumerate()
        {
            let sighash = sighash_cache
                .taproot_script_spend_signature_hash(
                    reveal_input_start_index + index,
                    &Prevouts::All(&prevouts),
                    TapLeafHash::from_script(reveal_script, LeafVersion::TapScript),
                    TapSighashType::Default,
                )
                .expect("failed to compute sighash");

            let secp = Secp256k1::new();
            let sig = secp.sign_schnorr(&sighash.into(), keypair);

            let witness = sighash_cache
                .witness_mut(reveal_input_start_index + index)
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

        debug!("Reveal tx after reveal part sign: {:?}", signed_reveal_tx);

        //sign the burn inscription input
        let signed_reveal_tx = if reveal_input_start_index > 0 {
            let signed_reveal_tx = sign_tx(&self.context, &signed_reveal_tx, &prevouts)?;
            debug!(
                "Reveal tx after burn part sign sign: {:?}",
                signed_reveal_tx
            );
            signed_reveal_tx
        } else {
            signed_reveal_tx
        };

        ctx.signed_reveal_tx = Some(signed_reveal_tx);
        Ok(())
    }

    fn sign(&self, ctx: &mut InscribeContext) -> Result<()> {
        // Sign the commit transaction
        self.sign_commit_tx(ctx)?;
        // Sign the reveal transaction
        self.sign_reveal_tx(ctx)?;
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

    async fn boardcaset_tx(&self, ctx: &mut InscribeContext) -> Result<InscribeOutput> {
        let total_fees = Self::calculate_fee(&ctx.commit_tx, &ctx.utxos)
            + Self::calculate_fee(&ctx.reveal_tx, &ctx.utxos);

        let origin_reveal_txid = ctx.reveal_tx.compute_txid();
        let reveal_input_count = ctx.reveal_scripts_to_sign.len();
        let inscriptions: Vec<_> = (0..reveal_input_count)
            .map(|index| InscriptionID::new(origin_reveal_txid, index as u32))
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

        let signed_commit_tx = ctx.signed_commit_tx.as_ref().expect("commit tx not signed");
        let signed_reveal_tx = ctx.signed_reveal_tx.as_ref().expect("reveal tx not signed");

        let commit_txid = match self
            .send_raw_transaction(signed_commit_tx, None, None)
            .await
        {
            Ok(txid) => txid,
            Err(err) => return Err(anyhow!("Failed to send commit transaction: {err}")),
        };

        let reveal_txid = match self.send_raw_transaction(signed_reveal_tx, None, ctx.total_burn_postage).await {
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

    pub async fn inscribe(&mut self) -> Result<InscribeOutput> {
        let mut ctx = self.prepare_context()?;

        self.build_commit(&mut ctx)?;
        self.build_reveal(&mut ctx).await?;
        self.update_fees(&mut ctx).await?;
        self.sign(&mut ctx)?;
        self.backup_keys(&mut ctx)?;
        let output = self.boardcaset_tx(&mut ctx).await?;
        Ok(output)
    }

    pub async fn send_raw_transaction(
        &self,
        tx: &Transaction,
        maxfeerate: Option<f64>,
        maxburnamount: Option<f64>,
    ) -> Result<bitcoin::Txid> {
        let client = self.context.get_client().await?;
        let txid = client
            .rooch
            .broadcast_bitcoin_tx(tx, maxfeerate, maxburnamount)
            .await?;
        Ok(Txid::from_str(&txid)?)
    }

    pub async fn get_operation_by_inscription_id(
        &self,
        inscription_id: InscriptionID,
    ) -> Result<Operation> {
        let ins_obj = self.get_inscription_object(inscription_id).await?;
        Operation::from_inscription(ins_obj.value.into())
    }

    async fn get_inscription_satpoint(&self, inscription_id: InscriptionID) -> Result<SatPoint> {
        let ins_obj = self.get_inscription_object(inscription_id).await?;
        Ok(ins_obj.location())
    }

    async fn get_inscription_object(
        &self,
        inscription_id: InscriptionID,
    ) -> Result<InscriptionObjectView> {
        let obj_id = inscription_id.object_id();
        let client = self.context.get_client().await?;
        let ins_obj = client
            .rooch
            .get_inscription_object(obj_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Inscription {} not found", inscription_id))?;
        Ok(ins_obj)
    }

    async fn load_generator(&self, generator: &str) -> Result<Box<dyn Generator>> {
        // generator: "/inscription/inscriptioin_id"
        let path = generator.split('/').collect::<Vec<&str>>();
        if path.len() != 3 {
            bail!("Invalid generator path: {:?}", generator);
        }
        let inscription_id = InscriptionID::from_str(path[2])?;
        let operation = self.get_operation_by_inscription_id(inscription_id).await?;
        let mint_record = operation
            .as_mint()
            .ok_or_else(|| anyhow!("Operation is not mint: {:?}", operation))?;
        if mint_record.sft.tick != GENERATOR_TICK {
            bail!("Invalid generator tick: {:?}", mint_record.sft.tick);
        }
        let content = mint_record
            .sft
            .content
            .as_ref()
            .ok_or_else(|| anyhow!("No content in generator mint record: {:?}", mint_record))?;
        ensure!(
            content.content_type == CONTENT_TYPE,
            "Invalid generator content type: {:?}",
            content.content_type
        );
        let wasm_bytecode = &content.body;
        Ok(Box::new(WASMGenerator::new(wasm_bytecode.clone())))
    }
}

//TODO migrate this function and `bitcoin::sign_psbt` to KeyStore in the future
pub fn sign_tx(
    context: &WalletContext,
    unsigned_tx: &Transaction,
    input_utxos: &[TxOut],
) -> Result<Transaction> {
    let secp = Secp256k1::new();
    let mut signed_inputs: Vec<(usize, TxIn)> = Vec::with_capacity(unsigned_tx.input.len());
    let mut sighash_cache = SighashCache::new(unsigned_tx);

    for (input_index, (input, utxo)) in unsigned_tx.input.iter().zip(input_utxos.iter()).enumerate()
    {
        if !input.witness.is_empty() || !input.script_sig.is_empty() {
            debug!("Skipping input {} as it is already signed", input_index);
            continue;
        }

        let script_pubkey = &utxo.script_pubkey;
        let bitcoin_address = BitcoinAddress::from(script_pubkey);

        let rooch_address = bitcoin_address.to_rooch_address();

        let keypair = context.get_key_pair(&rooch_address)?;

        let message: Message = if script_pubkey.is_p2pkh() {
            sighash_cache
                .legacy_signature_hash(input_index, script_pubkey, EcdsaSighashType::All.to_u32())?
                .into()
        } else if script_pubkey.is_p2wpkh() {
            sighash_cache
                .p2wpkh_signature_hash(
                    input_index,
                    script_pubkey,
                    utxo.value,
                    EcdsaSighashType::All,
                )?
                .into()
        } else if script_pubkey.is_p2wsh() {
            sighash_cache
                .p2wsh_signature_hash(
                    input_index,
                    script_pubkey,
                    utxo.value,
                    EcdsaSighashType::All,
                )?
                .into()
        } else if script_pubkey.is_p2tr() {
            sighash_cache
                .taproot_key_spend_signature_hash(
                    input_index,
                    &bitcoin::sighash::Prevouts::All(input_utxos),
                    TapSighashType::Default,
                )?
                .into()
        } else {
            return Err(anyhow::anyhow!(
                "Unsupported script type for input {}",
                input_index
            ));
        };

        let kp = keypair.secp256k1_keypair().ok_or_else(|| {
            anyhow::anyhow!("Failed to get private key for account {}", rooch_address)
        })?;

        let mut signed_input = input.clone();

        if script_pubkey.is_p2tr() {
            //The taproot key spend needs to be signed with the tweaked key
            let kp = Keypair::from_secret_key(&secp, &kp.secret_key())
                .tap_tweak(&secp, None)
                .to_inner();
            let signature = secp.sign_schnorr(&message, &kp);
            signed_input.witness = bitcoin::Witness::from_slice(&[signature.as_ref()]);
        } else {
            let signature = secp.sign_ecdsa(&message, &kp.secret_key());
            let mut sig_with_hashtype = signature.serialize_der().to_vec();
            sig_with_hashtype.push(EcdsaSighashType::All as u8);

            if script_pubkey.is_p2wpkh() {
                let pubkey = kp.public_key();
                signed_input.witness = bitcoin::Witness::from_slice(&[
                    &sig_with_hashtype,
                    &pubkey.serialize().to_vec(),
                ]);
            } else {
                let pubkey = kp.public_key();
                let buf = PushBytesBuf::try_from(sig_with_hashtype)?;
                signed_input.script_sig = bitcoin::Script::builder()
                    .push_slice(&buf)
                    .push_slice(pubkey.serialize())
                    .into_script();
            }
        }
        signed_inputs.push((input_index, signed_input));
    }

    let mut signed_tx = unsigned_tx.clone();
    for (index, signed_input) in signed_inputs {
        signed_tx.input[index] = signed_input;
    }

    Ok(signed_tx)
}

// #[cfg(test)]
// mod tests{
//     use super::*;

//     fn check_inscription_parse(tx: Transaction){

//     }

//     #[test]
//     fn test_generator(){
//         let wallet_context =
//         let output = Inscriber::new(context, self.inscribe_options)
//             .await?
//             .with_generator(self.name, self.generator)
//             .await?
//     }
// }
