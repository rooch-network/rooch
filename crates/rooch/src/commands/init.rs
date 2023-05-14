// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::config::{rooch_config_dir, Config, RoochConfig, ROOCH_CONFIG, ROOCH_KEYSTORE_FILENAME};
use clap::Parser;
use rooch_key::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use rooch_types::account::SignatureScheme::ED25519;
use rooch_types::cli::{CliError, CliResult};
use std::path::{Path, PathBuf};

// TODO: support server config
#[derive(Parser)]
pub struct Init {
    /// Sets the file storing the state of our user accounts (an empty one will be created if missing)
    #[clap(long = "rooch.config")]
    config: Option<PathBuf>,
}

impl Init {
    pub async fn execute(self) -> CliResult<()> {
        let config_path = self.config.unwrap_or(
            rooch_config_dir()
                .map_err(CliError::from)?
                .join(ROOCH_CONFIG),
        );
        prompt_if_no_config(&config_path)
            .await
            .map_err(CliError::from)?;
        Ok(())
    }
}

// Prompt user for connect to devnet fullnode if config does not exist.
async fn prompt_if_no_config(conf_path: &Path) -> Result<(), anyhow::Error> {
    let keystore_path = conf_path
        .parent()
        .unwrap_or(&rooch_config_dir()?)
        .join(ROOCH_KEYSTORE_FILENAME);
    let mut keystore = Keystore::from(FileBasedKeystore::new(&keystore_path)?);

    let (new_address, phrase, scheme) = keystore.generate_and_add_new_key(ED25519, None, None)?;

    println!(
        "Generated new keypair for address with scheme {:?} [{new_address}]",
        scheme.to_string()
    );
    println!("Secret Recovery Phrase : [{phrase}]");

    RoochConfig {
        keystore,
        active_address: Some(new_address),
    }
    .persisted(conf_path)
    .save()?;

    Ok(())
}
