// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_core_types::account_address::AccountAddress;
use rooch_key::keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::{
    address::RoochAddress,
    authentication_key::AuthenticationKeyType,
    crypto::{BuiltinScheme, PublicKey},
    error::{RoochError, RoochResult},
};

use crate::cli_types::{CommandAction, WalletContextOptions};
use std::str::FromStr;

/// Update an address with a new keypair from scheme to rooch.keystore
#[derive(Debug, Parser)]
pub struct UpdateCommand {
    /// Rooch address in string format.
    #[clap(short = 'a', long = "address")]
    address: String,
    #[clap(short = 'm', long = "mnemonic-phrase")]
    mnemonic_phrase: String,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    /// Authentication key type. Select an authentication key type with the Rooch address (Bitcoin P2PKH authentication key address leading with "1" or Bitcoin P2SH authentication key address leading with "3")
    #[clap(short = 't', long = "authentication-key-type", arg_enum)]
    pub authentication_key_type: Option<AuthenticationKeyType>,
    /// Command line input of crypto schemes (ed25519, multi-ed25519, ecdsa, ecdsa-recoverable or schnorr)
    #[clap(short = 's', long = "scheme", arg_enum)]
    pub crypto_schemes: BuiltinScheme,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for UpdateCommand {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        println!("{:?}", self.mnemonic_phrase);

        let mut context = self.context_options.build().await?;

        match BuiltinScheme::from_flag_byte(self.crypto_schemes.flag()) {
            Ok(scheme) => {
                let existing_address =
                    RoochAddress::from_str(self.address.as_str()).map_err(|e| {
                        RoochError::CommandArgumentError(format!(
                            "Invalid Rooch address String: {}",
                            e
                        ))
                    })?;

                let public_key: PublicKey = context
                    .config
                    .keystore
                    .update_address_with_key_pair_from_scheme(
                        &existing_address,
                        self.mnemonic_phrase,
                        scheme,
                        None,
                    )
                    .map_err(|e| RoochError::UpdateAccountError(e.to_string()))?;

                println!(
                    "{}",
                    AccountAddress::from(existing_address).to_hex_literal()
                );
                println!(
                    "Generated a new keypair for an existing address {:?} on scheme {:?}",
                    existing_address,
                    scheme.to_owned()
                );

                // Get public key reference
                let public_key = public_key.as_ref().to_vec();

                // Get decimal prefix or version from the input address type
                let decimal_prefix_or_version: Option<u8> = self
                    .authentication_key_type
                    .map(|addr_type| addr_type.decimal_prefix_or_version());

                // Create MoveAction from scheme
                let action = scheme.create_rotate_authentication_key_action(
                    public_key,
                    decimal_prefix_or_version,
                )?;

                // Execute the Move call as a transaction
                let result = context
                    .sign_and_execute(existing_address, action, scheme)
                    .await?;
                context.assert_execute_success(result)
            }
            Err(error) => {
                return Err(RoochError::CommandArgumentError(format!(
                    "Invalid crypto scheme: {}",
                    error
                )))
            }
        }
    }
}
