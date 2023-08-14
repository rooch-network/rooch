// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{TransactionOptions, WalletContextOptions};
use clap::Parser;
use rooch_key::keystore::AccountKeystore;
use rooch_types::{
    address::RoochAddress,
    authentication_key::AuthenticationKey,
    crypto::BuiltinScheme,
    error::{RoochError, RoochResult},
    framework::session_key::SessionScope,
};

/// Create a new session key on-chain
#[derive(Debug, Parser)]
pub struct CreateCommand {
    /// The scope of the session key, format: address::module_name::function_name.
    /// The module_name and function_name must be valid Move identifiers or '*'. `*` means any module or function.
    /// For example: 0x3::empty::empty
    #[clap(long)]
    pub scope: SessionScope,

    /// The expiration time of the session key, in seconds.
    #[clap(long, default_value = "3600")]
    pub expiration_time: u64,

    /// The max inactive interval of the session key, in seconds.
    #[clap(long, default_value = "3600")]
    pub max_inactive_interval: u64,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl CreateCommand {
    pub async fn execute(self) -> RoochResult<AuthenticationKey> {
        let mut context = self.context_options.build().await?;

        if self.tx_options.sender_account.is_none() {
            return Err(RoochError::CommandArgumentError(
                "--sender-account required".to_owned(),
            ));
        }
        let sender: RoochAddress = context
            .parse_account_arg(self.tx_options.sender_account.unwrap())?
            .into();

        let session_auth_key = context.config.keystore.generate_session_key(&sender)?;

        let session_scope = self.scope;

        let action =
            rooch_types::framework::session_key::SessionKeyModule::create_session_key_action(
                session_auth_key.as_ref().to_vec(),
                session_scope.clone(),
                self.expiration_time,
                self.max_inactive_interval,
            );

        println!("Generated new session key {session_auth_key} for address  [{sender}]",);

        let result = context
            .sign_and_execute(sender, action, BuiltinScheme::Ed25519)
            .await?;
        context.assert_execute_success(result)?;
        Ok(session_auth_key)
    }
}
