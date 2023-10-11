// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use crate::utils::read_line;
use async_trait::async_trait;
use clap::Parser;
use regex::Regex;
use rooch_config::config::Config;
use rooch_config::server_config::ServerConfig;
use rooch_config::{
    rooch_config_dir, ROOCH_CLIENT_CONFIG, ROOCH_KEYSTORE_FILENAME, ROOCH_SERVER_CONFIG,
};
use rooch_key::key_derive::hash_password;
use rooch_key::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use rooch_rpc_client::client_config::{ClientConfig, Env};
use rooch_types::error::RoochError;
use rooch_types::error::RoochResult;
use rpassword::prompt_password;
use std::fs;

/// Tool for init with rooch
#[derive(Parser)]
pub struct Init {
    /// Command line input of custom server URL
    #[clap(short = 's', long = "server-url")]
    pub server_url: Option<String>,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    /// Whether a non-empty password should be provided to rooch.keystore when it comes to the init command
    #[clap(long = "encrypt-keystore")]
    pub encrypt_keystore: Option<bool>,
}

#[async_trait]
impl CommandAction<()> for Init {
    async fn execute(self) -> RoochResult<()> {
        let config_path = match self.context_options.config_dir {
            Some(v) => {
                if !v.exists() {
                    fs::create_dir_all(v.clone())?
                }
                v
            }
            None => rooch_config_dir()?,
        };

        // Rooch client config init
        let client_config_path = config_path.join(ROOCH_CLIENT_CONFIG);

        let keystore_path = client_config_path
            .parent()
            .unwrap_or(&rooch_config_dir()?)
            .join(ROOCH_KEYSTORE_FILENAME);

        let keystore_result = FileBasedKeystore::new(&keystore_path);
        let mut keystore = match keystore_result {
            Ok(file_keystore) => Keystore::File(file_keystore),
            Err(error) => return Err(RoochError::GenerateKeyError(error.to_string())),
        };

        // Rooch server config init
        let server_config_path = config_path.join(ROOCH_SERVER_CONFIG);
        if !server_config_path.exists() {
            let server_config = ServerConfig::default();

            server_config
                .persisted(server_config_path.as_path())
                .save()?;

            println!(
                "Rooch server config file generated at {}",
                server_config_path.display()
            );
        } else {
            println!(
                "Rooch server config file already exists at {}",
                server_config_path.display()
            );
        }

        // Prompt user for connect to devnet fullnode if config does not exist.
        if !client_config_path.exists() {
            let env = match std::env::var_os("ROOCH_CONFIG_WITH_RPC_URL") {
                Some(v) => {
                    let chain_url: Vec<String> = v
                        .into_string()
                        .unwrap()
                        .split(',')
                        .map(|s| s.to_owned())
                        .collect();
                    Some(Env {
                        alias: "custom".to_string(),
                        rpc: chain_url[1].to_owned(),
                        ws: None,
                    })
                }

                None => {
                    println!(
                        "Creating config file [{:?}] with server and rooch native validator.",
                        client_config_path
                    );
                    let url = if self.server_url.is_none() {
                        String::new()
                    } else {
                        let address_and_port_regex =
                            Regex::new(r"^(https?://(?:\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}|localhost):\d{1,5})$")
                                .unwrap();
                        let url = self.server_url.unwrap();
                        print!("Rooch server URL: {:?} ", url);
                        // Check if input matches the regex pattern
                        if address_and_port_regex.is_match(&url) {
                            url
                        } else {
                            return Err(RoochError::CommandArgumentError("Invalid input format. Please provide a valid URL (e.g., http://0.0.0.0:50051).".to_owned()));
                        }
                    };
                    Some(if url.trim().is_empty() {
                        Env::default()
                    } else {
                        print!("Environment alias for [{url}] : ");
                        let alias = read_line()?;
                        let alias = if alias.trim().is_empty() {
                            "custom".to_string()
                        } else {
                            alias
                        };
                        print!("Environment ChainID for [{url}] : ");
                        Env {
                            alias,
                            rpc: url,
                            ws: None,
                        }
                    })
                }
            };

            if let Some(env) = env {
                let password = if self.encrypt_keystore.is_some() {
                    Some(prompt_password("Enter a password to encrypt the keys in the rooch keystore. Press return to have an empty value: ").unwrap_or_default())
                } else {
                    None
                };

                let result = keystore.generate_and_add_new_key(None, None, password.clone())?;
                println!("Generated new keypair for address [{}]", result.address);
                println!("Secret Recovery Phrase : [{}]", result.result.mnemonic);
                let dev_env = Env::new_dev_env();
                let active_env_alias = dev_env.alias.clone();

                let (password_hash, is_password_empty) = if password.is_none() {
                    ("$argon2id$v=19$m=19456,t=2,p=1$zc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc3Nzc0$RysE6tj+Zu0lLhtKJIedVHrKn9FspulS3vLj/UPaVvQ".to_owned(), true)
                } else {
                    (
                        hash_password(&result.result.encryption.nonce, password)?,
                        false,
                    )
                };

                let client_config = ClientConfig {
                    keystore_path,
                    password_hash: Some(password_hash),
                    is_password_empty,
                    envs: vec![env, dev_env],
                    active_address: Some(result.address),
                    // make dev env as default env
                    active_env: Some(active_env_alias),
                };

                client_config
                    .persisted(client_config_path.as_path())
                    .save()?;
            }

            println!(
                "Rooch client config file generated at {}",
                client_config_path.display()
            );
        } else {
            println!(
                "Rooch client config file already exists at {}",
                client_config_path.display()
            );
        }

        Ok(())
    }
}
