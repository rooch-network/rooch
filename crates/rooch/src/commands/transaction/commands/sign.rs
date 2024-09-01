// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{is_file_path, FileOutput, FileOutputData};
use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use moveos_types::module_binding::MoveFunctionCaller;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::{
    address::{ParsedAddress, RoochAddress},
    bitcoin::multisign_account::MultisignAccountModule,
    error::{RoochError, RoochResult},
    transaction::{
        authenticator::BitcoinAuthenticator, rooch::PartiallySignedRoochTransaction,
        RoochTransaction, RoochTransactionData,
    },
};
use std::{fs::File, io::Read, str::FromStr};

#[derive(Debug, Clone)]
pub enum SignInput {
    RoochTransactionData(RoochTransactionData),
    PartiallySignedRoochTransaction(PartiallySignedRoochTransaction),
}

impl FromStr for SignInput {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data_hex = if is_file_path(s) {
            //load hex from file
            let mut file = File::open(s).map_err(|e| {
                RoochError::CommandArgumentError(format!("Failed to open file: {}, err:{:?}", s, e))
            })?;
            let mut hex_str = String::new();
            file.read_to_string(&mut hex_str).map_err(|e| {
                RoochError::CommandArgumentError(format!("Failed to read file: {}, err:{:?}", s, e))
            })?;
            hex_str.strip_prefix("0x").unwrap_or(&hex_str).to_string()
        } else {
            s.strip_prefix("0x").unwrap_or(s).to_string()
        };
        let data_bytes = hex::decode(&data_hex).map_err(|e| {
            RoochError::CommandArgumentError(format!(
                "Failed to decode hex: {}, err:{:?}",
                data_hex, e
            ))
        })?;
        let input = match bcs::from_bytes(&data_bytes) {
            Ok(tx_data) => SignInput::RoochTransactionData(tx_data),
            Err(_) => {
                let psrt: PartiallySignedRoochTransaction = match bcs::from_bytes(&data_bytes) {
                    Ok(psrt) => psrt,
                    Err(_) => {
                        return Err(anyhow::anyhow!("Invalid tx data or psrt data"));
                    }
                };
                SignInput::PartiallySignedRoochTransaction(psrt)
            }
        };
        Ok(input)
    }
}

impl SignInput {
    pub fn sender(&self) -> RoochAddress {
        match self {
            SignInput::RoochTransactionData(tx_data) => tx_data.sender,
            SignInput::PartiallySignedRoochTransaction(psrt) => psrt.sender(),
        }
    }
}

pub enum SignOutput {
    SignedRoochTransaction(RoochTransaction),
    PartiallySignedRoochTransaction(PartiallySignedRoochTransaction),
}

impl SignOutput {
    pub fn is_finished(&self) -> bool {
        matches!(self, SignOutput::SignedRoochTransaction(_))
    }
}

impl From<SignOutput> for FileOutputData {
    fn from(val: SignOutput) -> Self {
        match val {
            SignOutput::SignedRoochTransaction(tx) => FileOutputData::SignedRoochTransaction(tx),
            SignOutput::PartiallySignedRoochTransaction(psrt) => {
                FileOutputData::PartiallySignedRoochTransaction(psrt)
            }
        }
    }
}

/// Get transactions by order
#[derive(Debug, clap::Parser)]
pub struct SignCommand {
    /// Input data to be used for signing
    /// Input can be a transaction data hex or a partially signed transaction data hex
    /// or a file path which contains transaction data or partially signed transaction data
    input: SignInput,

    /// The address of the signer when the transaction is a multisign account transaction
    /// If not specified, we will auto find the existing participants in the multisign account from the keystore
    #[clap(short = 's', long, value_parser=ParsedAddress::parse)]
    signer: Option<ParsedAddress>,

    /// The output file path for the signed transaction
    /// If not specified, the signed output will write to current directory.
    #[clap(long, short = 'o')]
    output: Option<String>,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,

    #[clap(flatten)]
    context: WalletContextOptions,
}

impl SignCommand {
    async fn sign(self) -> anyhow::Result<SignOutput> {
        let context = self.context.build_require_password()?;
        let client = context.get_client().await?;
        let multisign_account_module = client.as_module_binding::<MultisignAccountModule>();
        let sender = self.input.sender();
        let output = if multisign_account_module.is_multisign_account(sender.into())? {
            let threshold = multisign_account_module.threshold(sender.into())?;
            let mut psrt = match self.input {
                SignInput::RoochTransactionData(tx_data) => {
                    PartiallySignedRoochTransaction::new(tx_data, threshold)
                }
                SignInput::PartiallySignedRoochTransaction(psrt) => psrt,
            };
            match self.signer {
                Some(signer) => {
                    let signer = context.resolve_address(signer)?;
                    if !multisign_account_module.is_participant(sender.into(), signer)? {
                        return Err(anyhow::anyhow!(
                            "The signer address {} is not a participant in the multisign account",
                            signer
                        ));
                    }
                    let kp = context.get_key_pair(&signer.into())?;
                    let authenticator = BitcoinAuthenticator::sign(&kp, &psrt.data);
                    if psrt.contains_authenticator(&authenticator) {
                        return Err(anyhow::anyhow!(
                            "The signer has already signed the transaction"
                        ));
                    }
                    psrt.add_authenticator(authenticator)?;
                }
                None => {
                    let participants = multisign_account_module.participants(sender.into())?;
                    let mut has_participant = false;
                    for participant in participants.iter() {
                        if context
                            .keystore
                            .contains_address(&participant.participant_address.into())
                        {
                            has_participant = true;
                            let kp =
                                context.get_key_pair(&participant.participant_address.into())?;
                            let authenticator = BitcoinAuthenticator::sign(&kp, &psrt.data);
                            if psrt.contains_authenticator(&authenticator) {
                                continue;
                            }
                            psrt.add_authenticator(authenticator)?;
                        }
                    }
                    if !has_participant {
                        return Err(anyhow::anyhow!("No participant found in the multisign account from the keystore, participants: {:?}", participants));
                    }
                }
            }

            if psrt.is_fully_signed() {
                SignOutput::SignedRoochTransaction(psrt.try_into_rooch_transaction()?)
            } else {
                SignOutput::PartiallySignedRoochTransaction(psrt)
            }
        } else {
            let tx_data = match self.input {
                SignInput::RoochTransactionData(tx_data) => tx_data,
                SignInput::PartiallySignedRoochTransaction(_) => {
                    return Err(anyhow::anyhow!(
                        "Cannot sign a partially signed transaction with a single signer"
                    ))
                }
            };
            SignOutput::SignedRoochTransaction(context.sign_transaction(sender, tx_data).await?)
        };
        Ok(output)
    }
}

#[async_trait]
impl CommandAction<Option<FileOutput>> for SignCommand {
    async fn execute(self) -> RoochResult<Option<FileOutput>> {
        let json = self.json;
        let output = self.output.clone();
        let sign_output = self.sign().await?;
        let is_finished = sign_output.is_finished();

        let file_output_data = sign_output.into();
        let file_output = FileOutput::write_to_file(file_output_data, output)?;

        if !json {
            if is_finished {
                println!("Signed transaction is written to {:?}", file_output.path);
                println!(
                    "You can submit the transaction with `rooch tx submit {}`",
                    file_output.path
                );
            } else {
                println!(
                    "Partially signed transaction is written to {:?}",
                    file_output.path
                );
                println!("You can send the partially signed transaction to other signers, and sign it later with `rooch tx sign {}`", file_output.path);
            }
            Ok(None)
        } else {
            Ok(Some(file_output))
        }
    }
}
