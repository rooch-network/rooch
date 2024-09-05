// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use hex::ToHex;
use moveos_types::state::MoveState;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::{
    address::BitcoinAddress,
    error::RoochResult,
    framework::auth_payload::{AuthPayload, SignData, MESSAGE_INFO, MESSAGE_INFO_PREFIX},
    function_arg::{parse_function_arg, FunctionArg, ParsedFunctionId},
};

/// Sign a message with the bitcoin address
#[derive(Debug, Parser)]
pub struct SignCommand {
    // the address to be used
    #[clap(short = 'a', long = "btc-address")]
    bitcoin_address: BitcoinAddress,

    /// the message to be signed
    #[clap(short = 'm', long)]
    message: String,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<String>> for SignCommand {
    async fn execute(self) -> RoochResult<Option<String>> {
        let context = self.context_options.build_require_password()?;
        let password = context.get_password();

        let mut message_info = Vec::new();
        message_info.append(&mut MESSAGE_INFO.to_vec());
        message_info.append(&mut self.message.to_bytes());

        let sign_data = SignData::new_without_tx_hash(MESSAGE_INFO_PREFIX.to_vec(), message_info);
        let encoded_sign_data = sign_data.encode();

        let signature = context.keystore.sign_hashed(
            &self.bitcoin_address.to_rooch_address(),
            &encoded_sign_data,
            password,
        )?;

        let auth_payload = AuthPayload::new_without_tx_hash(sign_data, signature, self.bitcoin_address.to_string());
        let auth_payload_hex = auth_payload.to_bytes().encode_hex();

        if self.json {
            Ok(Some(auth_payload_hex))
        } else {
            println!(
                "Sign message succeeded with the auth payload hex: {:?}",
                auth_payload_hex
            );
            Ok(None)
        }
    }
}
