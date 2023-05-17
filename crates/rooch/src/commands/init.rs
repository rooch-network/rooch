// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;
use rooch_common::config::{
    rooch_config_path, Config, RoochConfig, ServerConfig, ROOCH_KEYSTORE_FILENAME,
};
use rooch_key::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use rooch_types::account::SignatureScheme::ED25519;
use rooch_types::cli::{CliError, CliResult, CommandAction};

#[derive(Parser)]
pub struct Init;

#[async_trait]
impl CommandAction<()> for Init {
    async fn execute(self) -> CliResult<()> {
        init().await.map_err(CliError::from)
    }
}

// Prompt user for connect to devnet fullnode if config does not exist.
pub async fn init() -> Result<(), anyhow::Error> {
    let server = ServerConfig::default();
    let conf_path = rooch_config_path()?;

    let keystore_path = conf_path.parent().unwrap().join(ROOCH_KEYSTORE_FILENAME);
    let mut keystore = Keystore::from(FileBasedKeystore::new(&keystore_path)?);

    let (new_address, phrase, scheme) = keystore.generate_and_add_new_key(ED25519, None, None)?;

    println!("server config: {:?}", server);

    println!(
        "Generated new keypair for address with scheme {:?} [{new_address}]",
        scheme.to_string()
    );
    println!("Secret Recovery Phrase : [{phrase}]");

    RoochConfig {
        keystore,
        active_address: Some(new_address),
        server: Some(server),
    }
    .persisted(conf_path.as_path())
    .save()?;

    Ok(())
}
