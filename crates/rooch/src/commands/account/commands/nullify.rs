// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
};
use moveos_types::{module_binding::ModuleBundle, move_types::FunctionId, transaction::MoveAction};
use rooch_rpc_api::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::framework::{
    ecdsa_k1_recoverable_validator::EcdsaK1RecoverableValidator,
    ecdsa_k1_validator::EcdsaK1Validator, ed25519_validator::Ed25519ValidatorModule,
    schnorr_validator::SchnorrValidator,
};
use std::fmt::Debug;

use async_trait::async_trait;
use rooch_key::keystore::AccountKeystore;
use rooch_types::{
    address::RoochAddress,
    crypto::BuiltinScheme,
    error::{RoochError, RoochResult},
};

use crate::cli_types::{CommandAction, WalletContextOptions};
use std::str::FromStr;

/// Nullify a keypair from a selected scheme with a Ed25519 generated address in rooch.keystore
#[derive(Debug, Parser)]
pub struct NullifyCommand {
    /// Rooch address in string format.
    #[clap(short = 'a', long = "address")]
    address: String,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    /// Command line input of crypto schemes (ed25519, multied25519, ecdsa, ecdsa-recoverable or schnorr)
    #[clap(short = 's', long = "scheme", arg_enum)]
    pub crypto_schemes: BuiltinScheme,
}

#[async_trait]
impl CommandAction<ExecuteTransactionResponseView> for NullifyCommand {
    async fn execute(self) -> RoochResult<ExecuteTransactionResponseView> {
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

                println!(
                    "{}",
                    AccountAddress::from(existing_address).to_hex_literal()
                );

                let (module_address, module_name) = match scheme {
                    BuiltinScheme::Ed25519 => (
                        Ed25519ValidatorModule::MODULE_ADDRESS,
                        Ed25519ValidatorModule::MODULE_NAME,
                    ),
                    BuiltinScheme::MultiEd25519 => todo!(),
                    BuiltinScheme::Ecdsa => (
                        EcdsaK1Validator::MODULE_ADDRESS,
                        EcdsaK1Validator::MODULE_NAME,
                    ),
                    BuiltinScheme::EcdsaRecoverable => (
                        EcdsaK1RecoverableValidator::MODULE_ADDRESS,
                        EcdsaK1RecoverableValidator::MODULE_NAME,
                    ),
                    BuiltinScheme::Schnorr => (
                        SchnorrValidator::MODULE_ADDRESS,
                        SchnorrValidator::MODULE_NAME,
                    ),
                };

                // Get validator struct
                let validator_struct_arg: Box<StructTag> =
                    scheme.create_validator_struct_tag(module_address, module_name.to_string())?;

                // Get the remove_authentication_key_entry_function
                let remove_authentication_key_entry_function = create_function_id(
                    module_address,
                    module_name.as_str(),
                    "remove_authentication_key_entry",
                );

                // Construct a Move call
                let action = MoveAction::new_function_call(
                    remove_authentication_key_entry_function,
                    vec![TypeTag::Struct(validator_struct_arg)],
                    vec![],
                );

                // Execute the Move call as a transaction
                let mut result = context
                    .sign_and_execute(existing_address, action, scheme)
                    .await?;
                result = context.assert_execute_success(result)?;

                // Remove keypair by scheme from Rooch key store after successfully executing transaction
                context
                    .config
                    .keystore
                    .nullify_address_with_key_pair_from_scheme(&existing_address, scheme)
                    .map_err(|e| RoochError::NullifyAccountError(e.to_string()))?;

                println!(
                    "Dropped a keypair from an existing address {:?} on scheme {:?}",
                    existing_address,
                    scheme.to_owned()
                );

                // Return transaction result
                Ok(result)
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

fn create_function_id(
    address: AccountAddress,
    module_name: &str,
    function_name: &str,
) -> FunctionId {
    FunctionId::new(
        ModuleId::new(address, Identifier::new(module_name).unwrap()),
        Identifier::new(function_name).unwrap(),
    )
}
