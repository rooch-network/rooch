// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use crate::utils::read_line;
use async_trait::async_trait;
use clap::Parser;
use regex::Regex;
use rooch_config::config::Config;
use rooch_config::{rooch_config_dir, ROOCH_CLIENT_CONFIG, ROOCH_KEYSTORE_FILENAME};
use rooch_key::keypair::KeyPairType;
use rooch_key::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use rooch_rpc_client::client_config::{ClientConfig, Env};
use rooch_types::address::RoochAddress;
use rooch_types::chain_id::RoochChainID;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::error::RoochError;
use rooch_types::error::RoochResult;
use std::fs;

/// Tool for init with rooch
#[derive(Parser)]
pub struct Init {
    /// Command line input of custom server URL
    #[clap(short = 's', long = "server-url")]
    pub server_url: Option<String>,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<String> for Init {
    async fn execute(self) -> RoochResult<String> {
        let client_config_path = match self.context_options.config_dir {
            Some(v) => {
                if !v.exists() {
                    fs::create_dir_all(v.clone())?
                }
                v.join(ROOCH_CLIENT_CONFIG)
            }
            None => rooch_config_dir()?.join(ROOCH_CLIENT_CONFIG),
        };
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
                        chain_id: chain_url[0].parse().unwrap(),
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
                        let chain_id = read_line()?;
                        let chain_id = chain_id
                            .trim()
                            .parse::<u64>()
                            .unwrap_or(RoochChainID::LOCAL.chain_id().id());
                        Env {
                            chain_id,
                            alias,
                            rpc: url,
                            ws: None,
                        }
                    })
                }
            };

            if let Some(env) = env {
                let keystore_path = client_config_path
                    .parent()
                    .unwrap_or(&rooch_config_dir()?)
                    .join(ROOCH_KEYSTORE_FILENAME);

                let keystore_result =
                    FileBasedKeystore::<RoochAddress, RoochKeyPair>::new(&keystore_path);
                let mut keystore = match keystore_result {
                    Ok(file_keystore) => Keystore::File(file_keystore),
                    Err(error) => return Err(RoochError::GenerateKeyError(error.to_string())),
                };

                let (new_address, phrase, key_pair_type) =
                    keystore.generate_and_add_new_key(KeyPairType::RoochKeyPairType, None, None)?;
                println!(
                    "Generated new keypair for address with type {:?} [{new_address}]",
                    key_pair_type.type_of()
                );
                println!("Secret Recovery Phrase : [{phrase}]");
                let alias = env.alias.clone();
                ClientConfig {
                    keystore,
                    envs: vec![env, Env::new_devnet_env(), Env::new_testnet_env()],
                    active_address: Some(new_address),
                    active_env: Some(alias),
                }
                .persisted(client_config_path.as_path())
                .save()?;
            }

            let message = format!(
                "Rooch config file generated at {}",
                client_config_path.display()
            );

            return Ok(message);
        }

        let message = format!(
            "Rooch config file already exists at {}",
            client_config_path.display()
        );

        Ok(message)
    }
}
