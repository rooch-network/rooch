// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use move_core_types::{
    account_address::AccountAddress,
    effects::Op,
    ident_str,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag, StructTag},
    parser::parse_type_tag,
};
use moveos_types::{transaction::TransactionOutput, module_binding::ModuleBundle};
use moveos_types::{
    move_types::FunctionId,
    transaction::{MoveAction, MoveOSTransaction},
};
use once_cell::sync::Lazy;
use rooch_framework::{ROOCH_FRAMEWORK_ADDRESS, bindings::{ed25519_validator::Ed25519Validator, ecdsa_k1_validator::EcdsaK1Validator, schnorr_validator::SchnorrValidator}};
use rooch_key::keystore::{AccountKeystore, Keystore};
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_types::{
    address::RoochAddress,
    crypto::{BuiltinScheme, PublicKey},
    error::{RoochError, RoochResult},
    transaction::{
        authenticator::{Authenticator, BuiltinAuthenticator},
        rooch::{RoochTransaction, RoochTransactionData},
    },
};
use move_core_types::value::MoveValue;

use crate::cli_types::{CommandAction, WalletContextOptions};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

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
    /// Command line input of crypto schemes (ed25519, multied25519, ecdsa, or schnorr)
    #[clap(short = 's', long = "scheme", arg_enum)]
    pub crypto_schemes: BuiltinScheme,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for UpdateCommand {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
        println!("{:?}", self.mnemonic_phrase);

        let mut context = self.context_options.build().await?;

        match BuiltinScheme::from_flag_byte(&self.crypto_schemes.flag()) {
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
                    "Generated a new keypair for an existing address on scheme {:?} [{existing_address}]",
                    scheme.to_owned()
                );

                let validator_struct_arg: Box<StructTag> = match scheme {
                    BuiltinScheme::Ed25519 => Ok(Box::new(StructTag {
                        address: Ed25519Validator::MODULE_ADDRESS,
                        module: Identifier::new(Ed25519Validator::MODULE_NAME).unwrap(),
                        name: Identifier::new(Ed25519Validator).unwrap(),
                        type_params: vec![],
                    })),
                    BuiltinScheme::Ecdsa => Ok(Box::new(StructTag {
                        address: EcdsaK1Validator::MODULE_ADDRESS,
                        module: Identifier::new(EcdsaK1Validator::MODULE_NAME).unwrap(),
                        name: Identifier::new(EcdsaK1Validator).unwrap(),
                        type_params: vec![],
                    })),
                    BuiltinScheme::Schnorr => Ok(Box::new(StructTag {
                        address: SchnorrValidator::MODULE_ADDRESS,
                        module: Identifier::new(SchnorrValidator::MODULE_NAME).unwrap(),
                        name: Identifier::new(SchnorrValidator).unwrap(),
                        type_params: vec![],
                    })),
                    _ => RoochError::CommandArgumentError("Validator for this scheme is not implemented".to_owned()),
                }?;

                println!(validator_struct_arg);
                // let move_value = MoveValue::Signer(AccountAddress::from(existing_address));
                // let signer = move_value
                // .simple_serialize()
                // .expect("serialize signer should success");
                let public_key_bytes_vec = public_key.as_ref().to_vec();
                println!(public_key_bytes_vec);

                let rotate_authentication_key_function = ROTATE_AUTHENTICATION_KEY_FUNCTION.clone();
                let action = MoveAction::new_function_call(
                    rotate_authentication_key_function,
                    vec![TypeTag::Struct(validator_struct_arg)],
                    vec![bcs::to_bytes(&existing_address).unwrap(), public_key_bytes_vec],
                );

                context
                    .sign_and_execute(existing_address, action, scheme)
                    .await
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

static ROTATE_AUTHENTICATION_KEY_FUNCTION: Lazy<FunctionId> = Lazy::new(|| {
    FunctionId::new(
        ModuleId::new(ROOCH_FRAMEWORK_ADDRESS, Identifier::new("account_authentication").unwrap()),
        Identifier::new("rotate_authentication_key").unwrap(),
    )
});
