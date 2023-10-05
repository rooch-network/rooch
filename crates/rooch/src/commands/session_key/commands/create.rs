// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{TransactionOptions, WalletContextOptions};
use clap::Parser;
use moveos_types::module_binding::MoveFunctionCaller;
use rooch_key::keystore::AccountKeystore;
use rooch_types::{
    address::RoochAddress,
    error::{RoochError, RoochResult},
    framework::session_key::{SessionKey, SessionKeyModule, SessionScope},
    keypair_type::KeyPairType,
};

/// Create a new session key on-chain
#[derive(Debug, Parser)]
pub struct CreateCommand {
    /// The scope of the session key, format: address::module_name::function_name.
    /// The module_name and function_name must be valid Move identifiers or '*'. `*` means any module or function.
    /// For example: 0x3::empty::empty
    #[clap(long)]
    pub scope: SessionScope,

    /// The max inactive interval of the session key, in seconds.
    /// If the max_inactive_interval is 0, the session key will never expire.
    #[clap(long, default_value = "3600")]
    pub max_inactive_interval: u64,

    /// Whether a password should be provided
    #[clap(long = "password")]
    password_required: Option<bool>,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl CreateCommand {
    pub async fn execute(self) -> RoochResult<SessionKey> {
        let mut context = self.context_options.build().await?;

        let password = if self.password_required == Some(false) {
            // Use an empty password if not required
            String::new()
        } else {
            // Prompt for a password if required
            rpassword::prompt_password("Enter a password to encrypt the keys in the rooch keystore. Press return to have an empty value: ").unwrap()
        };
        println!("Your password is {}", password);

        if self.tx_options.sender_account.is_none() {
            return Err(RoochError::CommandArgumentError(
                "--sender-account required".to_owned(),
            ));
        }
        let sender: RoochAddress = context
            .parse_account_arg(self.tx_options.sender_account.unwrap())?
            .into();

        let session_auth_key = context
            .keystore
            .generate_session_key(&sender, Some(password.clone()))?;

        let session_scope = self.scope;

        let action =
            rooch_types::framework::session_key::SessionKeyModule::create_session_key_action(
                session_auth_key.as_ref().to_vec(),
                session_scope.clone(),
                self.max_inactive_interval,
            );

        println!("Generated new session key {session_auth_key} for address [{sender}]",);

        let result = context
            .sign_and_execute(
                sender,
                action,
                KeyPairType::RoochKeyPairType,
                Some(password),
            )
            .await?;
        context.assert_execute_success(result)?;
        let client = context.get_client().await?;
        let session_key_module = client.as_module_binding::<SessionKeyModule>();
        let session_key = session_key_module
            .get_session_key(sender.into(), &session_auth_key)?
            .ok_or_else(|| {
                RoochError::ViewFunctionError(format!(
                    "Failed to get session key via {}",
                    session_auth_key
                ))
            })?;
        Ok(session_key)
    }
}
