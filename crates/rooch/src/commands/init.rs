// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Ok;
use clap::Parser;
use move_package::BuildConfig;
use moveos_common::config::{rooch_config_dir, ROOCH_CONFIG, ROOCH_KEYSTORE_FILENAME};
use moveos_common::keys::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use std::path::{Path, PathBuf};
use moveos_types::crypto::SignatureScheme::ED25519;

// TODO: support server config
#[derive(Parser)]
pub struct Init {
    /// Sets the file storing the state of our user accounts (an empty one will be created if missing)
    #[clap(long = "config")]
    config: Option<PathBuf>,
}

impl Init {
    pub async fn execute(self) -> anyhow::Result<()> {
        let config_path = self
            .config
            .unwrap_or(rooch_config_dir()?.join(ROOCH_CONFIG));
        prompt_if_no_config(&config_path).await?;
        Ok(())
    }
}

// Prompt user for connect to devnet fullnode if config does not exist.
async fn prompt_if_no_config(conf_path: &Path) -> Result<(), anyhow::Error> {
    if (!conf_path.exists()) {
        let keystore_path = conf_path
            .parent()
            .unwrap_or(&rooch_config_dir()?)
            .join(ROOCH_KEYSTORE_FILENAME);
        let mut keystore = Keystore::from(FileBasedKeystore::new(&keystore_path)?);

        let (new_address, phrase, scheme) =
            keystore.generate_and_add_new_key(ED25519, None, None)?;
        println!(
            "Generated new keypair for address with scheme {:?} [{new_address}]",
            scheme.to_string()
        );
        println!("Secret Recovery Phrase : [{phrase}]");
//        let alias = env.alias.clone();
//        SuiClientConfig {
//            keystore,
//            envs: vec![env],
//            active_address: Some(new_address),
//            active_env: Some(alias),
//        }
//        .persisted(wallet_conf_path)
//        .save()?;
    }

    Ok(())
}
