// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, TransactionOptions, WalletContextOptions};
use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use moveos_types::module_binding::MoveFunctionCaller;
use rooch_rpc_api::jsonrpc_types::BytesView;
use rooch_types::{
    address::RoochAddress,
    bitcoin::multisign_account::{self, MultisignAccountModule},
    error::RoochResult,
};
use serde::{Deserialize, Serialize};

/// Create a new multisigin account on-chain.
#[derive(Debug, Parser)]
pub struct CreateMultisignCommand {
    /// Public keys of the participants
    #[clap(long = "public-keys", short = 'p', required = true)]
    pub public_keys: Vec<String>,

    ///Threshold for multisign account
    #[clap(long = "threshold", short = 't')]
    pub threshold: u64,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInfoView {
    pub participant_address: RoochAddress,
    pub participant_bitcoin_address: String,
    pub public_key: BytesView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisignAccountOutput {
    pub multisign_address: RoochAddress,
    pub multisign_bitcoin_address: String,
    pub participants: Vec<ParticipantInfoView>,
}

#[async_trait]
impl CommandAction<Option<MultisignAccountOutput>> for CreateMultisignCommand {
    async fn execute(self) -> RoochResult<Option<MultisignAccountOutput>> {
        let context = self.context_options.build_require_password()?;

        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();
        let bitcoin_network = context.get_bitcoin_network().await?;

        let client = context.get_client().await?;
        let multisign_account_module = client.as_module_binding::<MultisignAccountModule>();

        let threshold = self.threshold;
        let public_keys = self
            .public_keys
            .iter()
            .map(|s| Ok(hex::decode(s.strip_prefix("0x").unwrap_or(s))?))
            .collect::<Result<Vec<_>>>()?;

        let multisign_bitcoin_address =
            multisign_account::generate_multisign_address(threshold as usize, public_keys.clone())?;

        //Build the transaction and create the multisign account on-chain
        let action =
            MultisignAccountModule::initialize_multisig_account_action(self.threshold, public_keys);

        let multisign_address = multisign_bitcoin_address.to_rooch_address();

        let is_onchain = multisign_account_module.is_multisign_account(multisign_address.into())?;

        if !is_onchain {
            let tx_data = context
                .build_tx_data(sender, action, self.tx_options.max_gas_amount)
                .await?;
            let signed_tx = context.sign_transaction(sender, tx_data)?;
            let result = context.execute(signed_tx).await?;
            context.assert_execute_success(result)?;
        }

        let participants = multisign_account_module.participants(multisign_address.into())?;

        let output: MultisignAccountOutput = MultisignAccountOutput {
            multisign_address,
            multisign_bitcoin_address: multisign_bitcoin_address
                .format(bitcoin_network)
                .expect("format multisign address should success"),
            participants: participants
                .into_iter()
                .map(|p| ParticipantInfoView {
                    participant_address: p.participant_address.into(),
                    participant_bitcoin_address: p
                        .participant_bitcoin_address
                        .format(bitcoin_network)
                        .expect("format participant address should success"),
                    public_key: p.public_key.into(),
                })
                .collect(),
        };
        if self.json {
            Ok(Some(output))
        } else {
            println!("MulitsignAddress: {}", output.multisign_address);
            println!(
                "Multisign Bitcoin Address: {}",
                output.multisign_bitcoin_address
            );
            println!("Threshold: {}", self.threshold);
            println!("Participants: {}", output.participants.len());
            //TODO make a table
            for (idx, participant) in output.participants.iter().enumerate() {
                println!(
                    "Participant {} Address: {}",
                    idx, participant.participant_address
                );
                println!(
                    "Participant {} Bitcoin Address: {}",
                    idx, participant.participant_bitcoin_address
                );
                println!("Participant {} Public Key: {}", idx, participant.public_key);
            }
            Ok(None)
        }
    }
}
