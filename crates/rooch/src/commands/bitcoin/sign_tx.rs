// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cli_types::{CommandAction, FileOrHexInput, WalletContextOptions},
    commands::bitcoin::{FileOutput, FileOutputData},
};
use anyhow::bail;
use anyhow::Result;
use async_trait::async_trait;
use bitcoin::{
    key::{Keypair, Secp256k1, TapTweak},
    sighash::{Prevouts, SighashCache},
    Psbt, TapLeafHash, TapSighashType, Witness,
};
use clap::Parser;
use moveos_types::module_binding::MoveFunctionCaller;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_rpc_client::{wallet_context::WalletContext, Client};
use rooch_types::{
    address::{BitcoinAddress, ParsedAddress, RoochAddress},
    bitcoin::multisign_account::MultisignAccountModule,
    error::{RoochError, RoochResult},
};
use tracing::debug;

#[derive(Debug, Parser)]
pub struct SignTx {
    /// The input psbt file path or hex string
    input: FileOrHexInput,

    /// The address of the signer when the transaction is a multisign account transaction
    /// If not specified, we will auto find the existing participants in the multisign account from the keystore
    #[clap(short = 's', long)]
    signer: Option<ParsedAddress>,

    /// The output file path
    /// If not provided, the file will be written to temp directory
    #[clap(long)]
    output_file: Option<String>,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[derive(Debug, Clone)]
pub enum SignOutput {
    Psbt(Psbt),
    Tx(bitcoin::Transaction),
}

#[async_trait]
impl CommandAction<FileOutput> for SignTx {
    async fn execute(self) -> RoochResult<FileOutput> {
        let context = self.context_options.build_require_password()?;
        let client = context.get_client().await?;

        let psbt = Psbt::deserialize(&self.input.data)?;
        debug!("psbt before sign: {:?}", psbt);
        let output = sign_psbt(psbt, self.signer, &context, &client).await?;
        debug!("sign output: {:?}", output);

        let file_output_data = match output {
            SignOutput::Psbt(psbt) => FileOutputData::Psbt(psbt),
            SignOutput::Tx(tx) => FileOutputData::Tx(tx),
        };
        let output = FileOutput::write_to_file(file_output_data, self.output_file)?;
        Ok(output)
    }
}

pub(crate) async fn sign_psbt(
    mut psbt: Psbt,
    signer: Option<ParsedAddress>,
    context: &WalletContext,
    client: &Client,
) -> Result<SignOutput, anyhow::Error> {
    let secp = Secp256k1::new();

    let signer = match signer {
        Some(signer) => Some(context.resolve_bitcoin_address(signer).await?),
        None => None,
    };

    let multisign_account_module = client.as_module_binding::<MultisignAccountModule>();

    let spend_utxos = (0..psbt.inputs.len())
        .map(|i| psbt.spend_utxo(i).ok().cloned())
        .collect::<Vec<_>>();

    if !spend_utxos.iter().all(Option::is_some) {
        bail!("Missing spend utxo");
    }

    let all_spend_utxos = spend_utxos.into_iter().flatten().collect::<Vec<_>>();
    let prevouts = Prevouts::All(&all_spend_utxos);

    let mut sighash_cache = SighashCache::new(&psbt.unsigned_tx);

    for (idx, input) in psbt.inputs.iter_mut().enumerate() {
        if let Some(utxo) = input.witness_utxo.as_ref() {
            let addr = BitcoinAddress::from(&utxo.script_pubkey);
            let rooch_addr = addr.to_rooch_address();
            if multisign_account_module.is_multisign_account(rooch_addr.into())? {
                let account_info = client.rooch.get_multisign_account_info(rooch_addr).await?;
                debug!("Account info: {:?}", account_info);
                let (control_block, (multisig_script, leaf_version)) = input
                    .tap_scripts
                    .iter()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("No tap script found for input {}", idx))?;

                let tap_leaf_hash = TapLeafHash::from_script(multisig_script, *leaf_version);
                debug!("Tap leaf hash: {:?}", tap_leaf_hash);

                let hash_ty = TapSighashType::Default;

                let sighash = sighash_cache.taproot_script_spend_signature_hash(
                    idx,
                    &prevouts,
                    tap_leaf_hash,
                    hash_ty,
                )?;
                debug!("Calculated sighash: {:?}", sighash);
                for participant in account_info.participants.values() {
                    if let Some(signer) = &signer {
                        if signer != &participant.participant_bitcoin_address {
                            continue;
                        }
                    }
                    let participant_addr: RoochAddress = participant.participant_address.into();
                    if context.keystore.contains_address(&participant_addr) {
                        debug!("Signing for participant: {}", participant_addr);
                        let kp = context.get_key_pair(&participant_addr)?;
                        let our_pubkey = kp.public().xonly_public_key()?;

                        let sk = kp.secp256k1_secret_key().expect("should have secret key");
                        let key_pair = Keypair::from_secret_key(&secp, &sk);

                        let signature = secp.sign_schnorr(&sighash.into(), &key_pair);

                        input.tap_script_sigs.insert(
                            (our_pubkey, tap_leaf_hash),
                            bitcoin::taproot::Signature {
                                signature,
                                sighash_type: hash_ty,
                            },
                        );
                    }
                }

                //Try to finalize the psbt
                if input.tap_script_sigs.len() >= account_info.threshold as usize {
                    //TODO handle multiple tap_leaf case

                    //make sure the signature order same as the public key order
                    let mut ordered_signatures = vec![];
                    let mut x_only_public_keys = account_info
                        .participants
                        .values()
                        .iter()
                        .map(|p| p.x_only_public_key())
                        .collect::<Result<Vec<_>>>()?;
                    x_only_public_keys.sort();
                    //Becase the stack is LIFO, we need to reverse the order
                    x_only_public_keys.reverse();

                    debug!("Ordered public keys before sign: {:?}", x_only_public_keys);

                    for xonly_pubkey in x_only_public_keys {
                        if let Some(sig) =
                            input.tap_script_sigs.remove(&(xonly_pubkey, tap_leaf_hash))
                        {
                            ordered_signatures.push(sig.to_vec());
                        } else {
                            //insert empty signature to ensure the order
                            ordered_signatures.push(vec![]);
                        }
                    }

                    debug!("Collected signatures: {:?}", ordered_signatures);

                    let mut witness = Witness::new();
                    for sig in ordered_signatures {
                        witness.push(sig);
                    }

                    witness.push(multisig_script.as_bytes());
                    witness.push(control_block.serialize());

                    debug!("Final witness: {:?}", witness);
                    input.final_script_witness = Some(witness);
                }
            } else {
                let kp = context.get_key_pair(&rooch_addr)?;
                let sk = kp.secp256k1_secret_key().expect("should have secret key");

                let key_pair = Keypair::from_secret_key(&secp, &sk)
                    .tap_tweak(&secp, input.tap_merkle_root)
                    .to_inner();

                let sighash = sighash_cache.taproot_key_spend_signature_hash(
                    idx,
                    &prevouts,
                    TapSighashType::Default,
                )?;
                debug!("Calculated sighash: {:?}", sighash);

                let signature = secp.sign_schnorr(&sighash.into(), &key_pair);
                debug!("Created signature: {:?}", signature);
                let tap_key_sig = bitcoin::taproot::Signature {
                    signature,
                    sighash_type: TapSighashType::Default,
                };

                let witness = Witness::from_slice(&[tap_key_sig.to_vec()]);
                input.tap_key_sig = Some(tap_key_sig);
                input.final_script_witness = Some(witness);
            }
        }
    }

    let sign_output = if is_psbt_finalized(&psbt) {
        let tx = psbt.extract_tx().map_err(|e| {
            RoochError::CommandArgumentError(format!("Failed to extract tx from psbt: {}", e))
        })?;
        SignOutput::Tx(tx)
    } else {
        SignOutput::Psbt(psbt)
    };

    Ok(sign_output)
}

fn is_psbt_finalized(psbt: &Psbt) -> bool {
    psbt.inputs
        .iter()
        .all(|input| input.final_script_sig.is_some() || input.final_script_witness.is_some())
}
