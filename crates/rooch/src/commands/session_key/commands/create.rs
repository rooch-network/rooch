// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{TransactionOptions, WalletContextOptions};
use clap::Parser;
use moveos_types::module_binding::MoveFunctionCaller;
use moveos_types::move_std::ascii::MoveAsciiString;
use moveos_types::move_std::string::MoveString;
use rooch_key::key_derive::verify_password;
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_types::{
    address::RoochAddress,
    error::{RoochError, RoochResult},
    framework::session_key::{SessionKey, SessionKeyModule, SessionScope},
};
use rpassword::prompt_password;

/// Create a new session key on-chain
#[derive(Debug, Parser)]
pub struct CreateCommand {
    #[clap(long)]
    pub app_name: MoveString,
    #[clap(long)]
    pub app_url: MoveAsciiString,

    /// The scope of the session key, format: address::module_name::function_name.
    /// The module_name and function_name must be valid Move identifiers or '*'. `*` means any module or function.
    /// For example: 0x3::empty::empty
    #[clap(long)]
    pub scope: SessionScope,

    /// The max inactive interval of the session key, in seconds.
    /// If the max_inactive_interval is 0, the session key will never expire.
    #[clap(long, default_value = "3600")]
    pub max_inactive_interval: u64,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl CreateCommand {
    pub async fn execute(self) -> RoochResult<SessionKey> {
        let mut context = self.context_options.build()?;

        let sender: RoochAddress = context.resolve_address(self.tx_options.sender)?.into();

        let session_auth_key = if context.keystore.get_if_password_is_empty() {
            context.keystore.generate_session_key(&sender, None)?
        } else {
            let password =
                prompt_password("Enter the password to create a new key pair:").unwrap_or_default();
            let is_verified =
                verify_password(Some(password.clone()), context.keystore.get_password_hash())?;

            if !is_verified {
                return Err(RoochError::InvalidPasswordError(
                    "Password is invalid".to_owned(),
                ));
            }

            context
                .keystore
                .generate_session_key(&sender, Some(password))?
        };
        let session_scope = self.scope;

        let action =
            rooch_types::framework::session_key::SessionKeyModule::create_session_key_action(
                self.app_name,
                self.app_url,
                session_auth_key.as_ref().to_vec(),
                session_scope.clone(),
                self.max_inactive_interval,
            );

        println!("Generated new session key {session_auth_key} for address [{sender}]",);

        let result = if context.keystore.get_if_password_is_empty() {
            context.sign_and_execute(sender, action, None).await?
        } else {
            let password =
                prompt_password("Enter the password to create a new key pair:").unwrap_or_default();
            let is_verified =
                verify_password(Some(password.clone()), context.keystore.get_password_hash())?;

            if !is_verified {
                return Err(RoochError::InvalidPasswordError(
                    "Password is invalid".to_owned(),
                ));
            }

            context
                .sign_and_execute(sender, action, Some(password))
                .await?
        };
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
        context
            .keystore
            .binding_session_key(sender, session_key.clone())?;
        Ok(session_key)
    }
}
