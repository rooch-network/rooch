// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::types::{CommandAction, WalletContextOptions};
use crate::utils::read_line;
use async_trait::async_trait;
use clap::Parser;
use rooch_client::client_config::{ClientConfig, Env};
use rooch_config::{rooch_config_dir, Config, ROOCH_CLIENT_CONFIG, ROOCH_KEYSTORE_FILENAME};
use rooch_key::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use rooch_types::{crypto::BuiltinScheme, error::RoochResult};
use std::fs;

#[derive(Parser)]
pub struct Init {
    #[clap(short = 'y', long = "yes", default_value_t = true)]
    pub accept_defaults: bool,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<()> for Init {
    async fn execute(self) -> RoochResult<()> {
        let client_config_path = match self.context_options.config_dir {
            Some(v) => {
                if !v.exists() {
                    fs::create_dir_all(v.clone())?
                }
                v.join(ROOCH_CLIENT_CONFIG)
            }
            None => rooch_config_dir()?.join(ROOCH_CLIENT_CONFIG),
        };
        //         Prompt user for connect to devnet fullnode if config does not exist.
        if !client_config_path.exists() {
            let env = match std::env::var_os("ROOCH_CONFIG_WITH_RPC_URL") {
                Some(v) => Some(Env {
                    alias: "custom".to_string(),
                    rpc: v.into_string().unwrap(),
                    ws: None,
                }),
                None => {
                    if self.accept_defaults {
                        println!("Creating config file [{:?}] with default (local) server and ed25519 key scheme.", client_config_path);
                    } else {
                        print!(
                                "Config file [{:?}] doesn't exist, do you want to connect to a rooch server [y/N]?",
                                client_config_path
                            );
                    }
                    if self.accept_defaults
                        || matches!(read_line(), Ok(line) if line.trim().to_lowercase() == "y")
                    {
                        let url = if self.accept_defaults {
                            String::new()
                        } else {
                            print!(
                                "Rooch server URL (Defaults to Rooch Local if not specified) : "
                            );
                            read_line()?
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
                            Env {
                                alias,
                                rpc: url,
                                ws: None,
                            }
                        })
                    } else {
                        None
                    }
                }
            };

            if let Some(env) = env {
                let keystore_path = client_config_path
                    .parent()
                    .unwrap_or(&rooch_config_dir()?)
                    .join(ROOCH_KEYSTORE_FILENAME);
                let mut keystore = Keystore::from(FileBasedKeystore::new(&keystore_path)?);
                let key_scheme = if self.accept_defaults {
                    BuiltinScheme::Ed25519
                } else {
                    println!(
                        "Select key scheme to generate keypair (0 for ed25519, 1 for secp256k1):"
                    );
                    BuiltinScheme::from_flag(read_line()?.trim())?
                };
                let (new_address, phrase, scheme) =
                    keystore.generate_and_add_new_key(key_scheme, None, None)?;
                println!(
                    "Generated new keypair for address with scheme {:?} [{new_address}]",
                    scheme.to_string()
                );
                println!("Secret Recovery Phrase : [{phrase}]");
                let alias = env.alias.clone();
                ClientConfig {
                    keystore,
                    envs: vec![env],
                    active_address: Some(new_address),
                    active_env: Some(alias),
                }
                .persisted(client_config_path.as_path())
                .save()?;
            }
        }

        Ok(())
    }
}
