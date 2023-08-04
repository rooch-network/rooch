// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
};
use moveos_types::module_binding::ModuleBundle;
use moveos_types::{move_types::FunctionId, transaction::MoveAction};
use once_cell::sync::Lazy;
use rooch_framework::{
    bindings::{
        ecdsa_k1_recoverable_validator::EcdsaK1RecoverableValidator,
        ecdsa_k1_validator::EcdsaK1Validator, ed25519_validator::Ed25519Validator,
        schnorr_validator::SchnorrValidator,
    },
    ROOCH_FRAMEWORK_ADDRESS,
};
use rooch_key::keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::{
    address::RoochAddress,
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
    /// Command line input of crypto schemes (ed25519, multied25519, ecdsa, or schnorr)
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
                    "Generated a new keypair for an existing address on scheme {:?} [{existing_address}]",
                    scheme.to_owned()
                );

                let address = match scheme {
                    BuiltinScheme::Ed25519 => Ed25519Validator::MODULE_ADDRESS,
                    BuiltinScheme::MultiEd25519 => todo!(),
                    BuiltinScheme::Ecdsa => EcdsaK1Validator::MODULE_ADDRESS,
                    BuiltinScheme::EcdsaRecoverable => EcdsaK1RecoverableValidator::MODULE_ADDRESS,
                    BuiltinScheme::Schnorr => SchnorrValidator::MODULE_ADDRESS,
                };

                let module_name = match scheme {
                    BuiltinScheme::Ed25519 => Ed25519Validator::MODULE_NAME,
                    BuiltinScheme::MultiEd25519 => todo!(),
                    BuiltinScheme::Ecdsa => EcdsaK1Validator::MODULE_NAME,
                    BuiltinScheme::EcdsaRecoverable => EcdsaK1RecoverableValidator::MODULE_NAME,
                    BuiltinScheme::Schnorr => SchnorrValidator::MODULE_NAME,
                };

                let validator_struct_arg: Box<StructTag> =
                    scheme.create_validator_struct_tag(address, module_name.to_string())?;

                let signer = bcs::to_bytes(&existing_address).unwrap();
                let public_key_bytes_vec = public_key.as_ref().to_vec();

                let rotate_authentication_key_entry_function =
                    ROTATE_AUTHENTICATION_KEY_ENTRY_FUNCTION.clone();
                let action = MoveAction::new_function_call(
                    rotate_authentication_key_entry_function,
                    vec![TypeTag::Struct(validator_struct_arg)],
                    vec![signer, public_key_bytes_vec],
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

static ROTATE_AUTHENTICATION_KEY_ENTRY_FUNCTION: Lazy<FunctionId> = Lazy::new(|| {
    FunctionId::new(
        ModuleId::new(
            ROOCH_FRAMEWORK_ADDRESS,
            Identifier::new("account_authentication").unwrap(),
        ),
        Identifier::new("rotate_authentication_key_entry").unwrap(),
    )
});
