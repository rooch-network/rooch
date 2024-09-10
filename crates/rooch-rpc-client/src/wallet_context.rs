// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::client_config::{ClientConfig, DEFAULT_EXPIRATION_SECS};
use crate::Client;
use anyhow::{anyhow, Result};
use bitcoin::key::Secp256k1;
use bitcoin::psbt::{GetKey, KeyRequest};
use bitcoin::secp256k1::Signing;
use bitcoin::PrivateKey;
use move_core_types::account_address::AccountAddress;
use moveos_types::moveos_std::gas_schedule::GasScheduleConfig;
use moveos_types::transaction::MoveAction;
use rooch_config::config::{Config, PersistedConfig};
use rooch_config::{rooch_config_dir, ROOCH_CLIENT_CONFIG};
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::file_keystore::FileBasedKeystore;
use rooch_key::keystore::Keystore;
use rooch_rpc_api::jsonrpc_types::{
    DryRunTransactionResponseView, ExecuteTransactionResponseView, KeptVMStatusView, TxOptions,
};
use rooch_types::address::RoochAddress;
use rooch_types::address::{BitcoinAddress, ParsedAddress};
use rooch_types::bitcoin::network::Network;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::rooch_network::{BuiltinChainID, RoochNetwork};
use rooch_types::transaction::rooch::{RoochTransaction, RoochTransactionData};
use rooch_types::{addresses, crypto};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

#[derive(Debug)]
pub struct WalletContext {
    client: Arc<RwLock<Option<Client>>>,
    pub client_config: PersistedConfig<ClientConfig>,
    pub keystore: Keystore,
    pub address_mapping: BTreeMap<String, AccountAddress>,
    password: Option<String>,
}

pub type AddressMappingFn = Box<dyn Fn(&str) -> Option<AccountAddress> + Send + Sync>;

impl WalletContext {
    pub fn new(config_path: Option<PathBuf>) -> Result<Self, anyhow::Error> {
        let config_dir = config_path.unwrap_or(rooch_config_dir()?);
        let client_config_path = config_dir.join(ROOCH_CLIENT_CONFIG);
        let client_config: ClientConfig = PersistedConfig::read(&client_config_path).map_err(|err| {
            anyhow!(
                "Cannot open wallet config file at {:?}. Err: {err}, Use `rooch init` to configuration",
                client_config_path
            )
        })?;

        let mut client_config = client_config.persisted(&client_config_path);

        let keystore_result = FileBasedKeystore::load(&client_config.keystore_path);
        let keystore = match keystore_result {
            Ok(file_keystore) => Keystore::File(file_keystore),
            Err(error) => return Err(error),
        };

        let mut address_mapping = BTreeMap::new();
        address_mapping.extend(addresses::rooch_framework_named_addresses());

        //TODO support account name alias name.
        if let Some(active_address) = &client_config.active_address {
            let active_address = if !keystore.contains_address(active_address) {
                //The active address is not in the keystore, maybe the user reset the keystore.
                //We auto change the active address to the first address in the keystore.
                let first_address = keystore
                    .addresses()
                    .pop()
                    .ok_or_else(|| anyhow!("No address in the keystore"))?;
                info!("The active address {} is not in the keystore, auto change the active address to the first address in the keystore: {}", active_address, first_address);
                client_config.active_address = Some(first_address);
                client_config.save()?;
                first_address
            } else {
                *active_address
            };
            address_mapping.insert("default".to_string(), active_address.into());
        }

        Ok(Self {
            client: Default::default(),
            client_config,
            keystore,
            address_mapping,
            password: None,
        })
    }

    pub fn add_address_mapping(&mut self, name: String, address: RoochAddress) {
        self.address_mapping.insert(name, address.into());
    }

    pub fn address_mapping(&self) -> AddressMappingFn {
        let address_mapping = self.address_mapping.clone();
        Box::new(move |name| address_mapping.get(name).cloned())
    }

    pub fn resolve_address(&self, parsed_address: ParsedAddress) -> RoochResult<AccountAddress> {
        self.resolve_rooch_address(parsed_address)
            .map(|address| address.into())
    }

    pub fn resolve_rooch_address(
        &self,
        parsed_address: ParsedAddress,
    ) -> RoochResult<RoochAddress> {
        match parsed_address {
            ParsedAddress::Numerical(address) => Ok(address),
            ParsedAddress::Named(name) => self
                .address_mapping
                .get(&name)
                .cloned()
                .map(|address| address.into())
                .ok_or_else(|| {
                    RoochError::CommandArgumentError(format!("Unknown named address: {}", name))
                }),
            ParsedAddress::Bitcoin(address) => Ok(address.to_rooch_address()),
        }
    }

    pub async fn resolve_bitcoin_address(
        &self,
        parsed_address: ParsedAddress,
    ) -> RoochResult<BitcoinAddress> {
        match parsed_address {
            ParsedAddress::Bitcoin(address) => Ok(address),
            _ => {
                let address = self.resolve_rooch_address(parsed_address)?;
                let account = self.keystore.get_account(&address, self.password.clone())?;
                if let Some(account) = account {
                    let bitcoin_address = account.bitcoin_address;
                    Ok(bitcoin_address)
                } else {
                    let client = self.get_client().await?;
                    let bitcoin_address = client.rooch.resolve_bitcoin_address(address).await?;
                    bitcoin_address.ok_or_else(|| {
                        RoochError::CommandArgumentError(format!(
                            "Cannot resolve bitcoin address from {}",
                            address
                        ))
                    })
                }
            }
        }
    }

    /// Parse and resolve addresses from a map of name to address string
    pub fn parse_and_resolve_addresses(
        &self,
        addresses: BTreeMap<String, String>,
    ) -> RoochResult<BTreeMap<String, AccountAddress>> {
        addresses
            .into_iter()
            .map(|(key, value)| {
                let parsed_address = ParsedAddress::parse(value.as_str())?;
                let account_address = self.resolve_address(parsed_address)?;
                Ok((key, account_address))
            })
            .collect::<Result<BTreeMap<_, _>>>()
            .map_err(|e| RoochError::CommandArgumentError(e.to_string()))
    }

    pub async fn get_client(&self) -> Result<Client, anyhow::Error> {
        // TODO: Check version

        let read = self.client.read().await;

        Ok(if let Some(client) = read.as_ref() {
            client.clone()
        } else {
            drop(read);
            let client = self
                .client_config
                .get_active_env()?
                .create_rpc_client(Duration::from_secs(DEFAULT_EXPIRATION_SECS))
                .await?;

            self.client.write().await.insert(client).clone()
        })
    }

    pub async fn build_tx_data(
        &self,
        sender: RoochAddress,
        action: MoveAction,
        max_gas_amount: Option<u64>,
    ) -> RoochResult<RoochTransactionData> {
        let client = self.get_client().await?;
        let chain_id = client.rooch.get_chain_id().await?;
        let sequence_number = client
            .rooch
            .get_sequence_number(sender)
            .await
            .map_err(RoochError::from)?;
        log::debug!("use sequence_number: {}", sequence_number);
        //TODO max gas amount from cli option or dry run estimate
        let tx_data = RoochTransactionData::new(
            sender,
            sequence_number,
            chain_id,
            max_gas_amount.unwrap_or(GasScheduleConfig::CLI_DEFAULT_MAX_GAS_AMOUNT),
            action,
        );
        Ok(tx_data)
    }

    pub async fn sign(
        &self,
        sender: RoochAddress,
        action: MoveAction,
        password: Option<String>,
        max_gas_amount: Option<u64>,
    ) -> RoochResult<RoochTransaction> {
        let tx_data = self.build_tx_data(sender, action, max_gas_amount).await?;
        let tx = self.keystore.sign_transaction(&sender, tx_data, password)?;
        Ok(tx)
    }

    pub async fn sign_transaction(
        &self,
        signer: RoochAddress,
        tx_data: RoochTransactionData,
    ) -> RoochResult<RoochTransaction> {
        let tx = self
            .keystore
            .sign_transaction(&signer, tx_data, self.password.clone())?;
        Ok(tx)
    }

    pub async fn execute(
        &self,
        tx: RoochTransaction,
    ) -> RoochResult<ExecuteTransactionResponseView> {
        let client = self.get_client().await?;
        client
            .rooch
            .execute_tx(
                tx,
                Some(TxOptions {
                    with_output: true,
                    decode: true,
                }),
            )
            .await
            .map_err(|e| RoochError::TransactionError(e.to_string()))
    }

    pub async fn sign_and_execute(
        &self,
        sender: RoochAddress,
        action: MoveAction,
        password: Option<String>,
        max_gas_amount: Option<u64>,
    ) -> RoochResult<ExecuteTransactionResponseView> {
        let tx = self.sign(sender, action, password, max_gas_amount).await?;
        self.execute(tx).await
    }

    pub fn get_key_pair(&self, address: &RoochAddress) -> Result<RoochKeyPair> {
        self.keystore.get_key_pair(address, self.password.clone())
    }

    pub async fn dry_run(
        &self,
        tx: RoochTransactionData,
    ) -> RoochResult<DryRunTransactionResponseView> {
        let client = self.get_client().await?;
        client
            .rooch
            .dry_run_tx(tx)
            .await
            .map_err(|e| RoochError::DryRunTransactionError(e.to_string()))
    }

    pub fn assert_execute_success(
        &self,
        result: ExecuteTransactionResponseView,
    ) -> RoochResult<ExecuteTransactionResponseView> {
        if KeptVMStatusView::Executed != result.execution_info.status {
            Err(RoochError::TransactionError(format!(
                "Transaction execution failed: {:?}",
                result.execution_info.status
            )))
        } else {
            Ok(result)
        }
    }

    pub fn set_password(&mut self, password: Option<String>) {
        self.password = password;
    }

    pub fn get_password(&self) -> Option<String> {
        self.password.clone()
    }

    pub async fn get_rooch_network(&self) -> Result<RoochNetwork> {
        let client = self.get_client().await?;
        let chain_id = client.rooch.get_chain_id().await?;
        //TODO support custom chain id
        let builtin_chain_id = BuiltinChainID::try_from(chain_id)?;
        Ok(builtin_chain_id.into())
    }

    pub async fn get_bitcoin_network(&self) -> Result<Network> {
        let rooch_network = self.get_rooch_network().await?;
        let bitcoin_network = rooch_types::bitcoin::network::Network::from(
            rooch_network.genesis_config.bitcoin_network,
        );
        Ok(bitcoin_network)
    }
}

impl GetKey for WalletContext {
    type Error = anyhow::Error;

    fn get_key<C: Signing>(
        &self,
        key_request: KeyRequest,
        _secp: &Secp256k1<C>,
    ) -> Result<Option<PrivateKey>, Self::Error> {
        debug!("Get key for key_request: {:?}", key_request);
        let address = match key_request {
            KeyRequest::Pubkey(pubkey) => {
                let rooch_public_key = crypto::PublicKey::from_bitcoin_pubkey(&pubkey)?;
                rooch_public_key.rooch_address()?
            }
            KeyRequest::Bip32(_key_source) => {
                anyhow::bail!("BIP32 key source is not supported");
            }
            _ => anyhow::bail!("Unsupported key request: {:?}", key_request),
        };
        debug!("Get key for address: {:?}", address);
        let kp = self
            .keystore
            .get_key_pair(&address, self.password.clone())?;
        Ok(Some(PrivateKey::from_slice(
            kp.private(),
            bitcoin::Network::Bitcoin,
        )?))
    }
}
