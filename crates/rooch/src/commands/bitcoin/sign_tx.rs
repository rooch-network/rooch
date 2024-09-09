// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use crate::{
    cli_types::{CommandAction, WalletContextOptions},
    commands::transaction::commands::is_file_path,
};
use async_trait::async_trait;
use bitcoin::{consensus::Encodable, key::Secp256k1};
use clap::Parser;
use moveos_types::module_binding::MoveFunctionCaller;
use rooch_rpc_client::{wallet_context::WalletContext, Client};
use rooch_types::{
    address::BitcoinAddress,
    bitcoin::multisign_account::{self, MultisignAccountModule},
    error::RoochResult,
};

#[derive(Debug, Parser)]
pub struct SignTx {
    input: String,
    #[clap(long)]
    output_file: Option<String>,

    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<String> for SignTx {
    async fn execute(self) -> RoochResult<String> {
        let context = self.context_options.build_require_password()?;
        let client = context.get_client().await?;

        let bytes = if is_file_path(&self.input) {
            std::fs::read(self.input)?
        } else {
            let hex = self.input.strip_prefix("0x").unwrap_or(&self.input);
            hex::decode(hex)?
        };

        let psbt = bitcoin::Psbt::deserialize(&bytes)?;

        let tx = sign_tx(psbt, &context, &client).await?;
        let mut bytes = vec![];
        tx.consensus_encode(&mut bytes)?;
        let signed_tx_hex = hex::encode(bytes);
        Ok(signed_tx_hex)
    }
}

async fn sign_tx(
    mut psbt: bitcoin::Psbt,
    context: &WalletContext,
    client: &Client,
) -> Result<bitcoin::Transaction, anyhow::Error> {
    let secp = Secp256k1::new();

    let multisign_account_module = client.as_module_binding::<MultisignAccountModule>();

    let mut multisign_addresses = HashSet::new();
    for input in psbt.inputs.iter_mut() {
        if let Some(utxo) = input.witness_utxo.as_ref() {
            let addr = BitcoinAddress::from(&utxo.script_pubkey);
            let rooch_addr = addr.to_rooch_address();
            if multisign_account_module.is_multisign_account(rooch_addr.into())? {
                multisign_addresses.insert(rooch_addr);
            }
        }
    }

    for rooch_addr in multisign_addresses {
        let kp = context.get_key_pair(&rooch_addr)?;
        multisign_account::sign_taproot_multisig(&mut psbt, &kp)?;
    }

    psbt.sign(context, &secp)
        .map_err(|_e| anyhow::anyhow!("Sign psbt errror"))?;

    Ok(psbt.extract_tx()?)
}
