// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::client_config::{ClientConfig, DEFAULT_EXPIRATION_SECS};
use crate::Client;
use anyhow::{anyhow, Result};
use move_command_line_common::address::ParsedAddress;
use move_core_types::account_address::AccountAddress;
use moveos_types::moveos_std::gas_schedule::GasScheduleConfig;
use moveos_types::transaction::MoveAction;
use rooch_config::config::{Config, PersistedConfig};
use rooch_config::server_config::ServerConfig;
use rooch_config::{rooch_config_dir, ROOCH_CLIENT_CONFIG, ROOCH_SERVER_CONFIG};
use rooch_key::keystore::account_keystore::AccountKeystore;
use rooch_key::keystore::file_keystore::FileBasedKeystore;
use rooch_key::keystore::Keystore;
use rooch_rpc_api::jsonrpc_types::{ExecuteTransactionResponseView, KeptVMStatusView, TxOptions};
use rooch_types::address::RoochAddress;
use rooch_types::addresses;
use rooch_types::crypto::Signature;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::transaction::{
    authenticator::Authenticator,
    rooch::{RoochTransaction, RoochTransactionData},
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub struct WalletContext {
    client: Arc<RwLock<Option<Client>>>,
    pub client_config: PersistedConfig<ClientConfig>,
    pub server_config: PersistedConfig<ServerConfig>,
    pub keystore: Keystore,
    pub address_mapping: BTreeMap<String, AccountAddress>,
}

pub type AddressMappingFn = Box<dyn Fn(&str) -> Option<AccountAddress> + Send + Sync>;

impl WalletContext {
    pub fn new(config_path: Option<PathBuf>) -> Result<Self, anyhow::Error> {
        let config_dir = config_path.unwrap_or(rooch_config_dir()?);
        let client_config_path = config_dir.join(ROOCH_CLIENT_CONFIG);
        let server_config_path = config_dir.join(ROOCH_SERVER_CONFIG);
        let client_config: ClientConfig = PersistedConfig::read(&client_config_path).map_err(|err| {
            anyhow!(
                "Cannot open wallet config file at {:?}. Err: {err}, Use `rooch init` to configuration",
                client_config_path
            )
        })?;
        let server_config: ServerConfig = PersistedConfig::read(&server_config_path).map_err(|err| {
            anyhow!(
                "Cannot open server config file at {:?}. Err: {err}, Use `rooch init` to configuration",
                server_config_path
            )
        })?;

        let client_config = client_config.persisted(&client_config_path);
        let server_config = server_config.persisted(&server_config_path);

        let keystore_result = FileBasedKeystore::load(&client_config.keystore_path);
        let keystore = match keystore_result {
            Ok(file_keystore) => Keystore::File(file_keystore),
            Err(error) => return Err(error),
        };

        let mut address_mapping = BTreeMap::new();
        address_mapping.extend(addresses::rooch_framework_named_addresses());

        //TODO support account name alias name.
        if let Some(active_address) = client_config.active_address {
            address_mapping.insert("default".to_string(), AccountAddress::from(active_address));
        }

        Ok(Self {
            client: Default::default(),
            client_config,
            server_config,
            keystore,
            address_mapping,
        })
    }

    pub fn add_address_mapping(&mut self, name: String, address: AccountAddress) {
        self.address_mapping.insert(name, address);
    }

    pub fn address_mapping(&self) -> AddressMappingFn {
        let address_mapping = self.address_mapping.clone();
        Box::new(move |name| address_mapping.get(name).cloned())
    }

    pub fn resolve_address(&self, parsed_address: ParsedAddress) -> RoochResult<AccountAddress> {
        match parsed_address {
            ParsedAddress::Numerical(address) => Ok(address.into_inner()),
            ParsedAddress::Named(name) => {
                self.address_mapping.get(&name).cloned().ok_or_else(|| {
                    RoochError::CommandArgumentError(format!("Unknown named address: {}", name))
                })
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
                .create_rpc_client(Duration::from_secs(DEFAULT_EXPIRATION_SECS), None)
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
            max_gas_amount.unwrap_or(GasScheduleConfig::INITIAL_MAX_GAS_AMOUNT),
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
        let kp = self
            .keystore
            .get_key_pair_with_password(&sender, password)
            .ok()
            .ok_or_else(|| {
                RoochError::SignMessageError(format!(
                    "Cannot find encryption data for address: [{sender}]"
                ))
            })?;

        let tx_data = self.build_tx_data(sender, action, max_gas_amount).await?;
        let signature = Signature::new_hashed(tx_data.tx_hash().as_bytes(), &kp);
        Ok(RoochTransaction::new(
            tx_data,
            Authenticator::rooch(signature),
        ))
    }

    pub async fn execute(
        &self,
        tx: RoochTransaction,
    ) -> RoochResult<ExecuteTransactionResponseView> {
        let client = self.get_client().await?;
        client
            .rooch
            .execute_tx(tx, Some(TxOptions { with_output: true }))
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
}
