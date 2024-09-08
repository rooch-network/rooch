// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cli_types::{CommandAction, WalletContextOptions},
    commands::transaction::commands::is_file_path,
};
use async_trait::async_trait;
use bitcoin::{consensus::Encodable, key::Secp256k1};
use clap::Parser;
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::error::RoochResult;

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
        let _client = context.get_client().await?;

        let bytes = if is_file_path(&self.input) {
            std::fs::read(self.input)?
        } else {
            let hex = self.input.strip_prefix("0x").unwrap_or(&self.input);
            hex::decode(hex)?
        };

        let psbt = bitcoin::Psbt::deserialize(&bytes)?;

        let tx = sign_tx(psbt, &context)?;
        let mut bytes = vec![];
        tx.consensus_encode(&mut bytes)?;
        let signed_tx_hex = hex::encode(bytes);
        Ok(signed_tx_hex)
    }
}

fn sign_tx(
    mut psbt: bitcoin::Psbt,
    context: &WalletContext,
) -> Result<bitcoin::Transaction, anyhow::Error> {
    let secp = Secp256k1::new();
    psbt.sign(context, &secp)
        .map_err(|_e| anyhow::anyhow!("Sign psbt errror"))?;
    Ok(psbt.extract_tx()?)
}
