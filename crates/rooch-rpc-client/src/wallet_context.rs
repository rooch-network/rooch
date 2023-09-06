// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::client_config::{ClientConfig, DEFAULT_EXPIRATION_SECS};
use crate::Client;
use anyhow::anyhow;
use ethers::types::{Bytes, OtherFields, Transaction, TransactionReceipt, H256, U256, U64, BlockNumber};
use fastcrypto::hash::Keccak256;
use fastcrypto::secp256k1::recoverable::Secp256k1RecoverableKeyPair;
use fastcrypto::traits::RecoverableSigner;
use move_core_types::account_address::AccountAddress;
use moveos_types::gas_config::GasConfig;
use moveos_types::transaction::MoveAction;
use rooch_config::config::{Config, PersistedConfig};
use rooch_config::{rooch_config_dir, ROOCH_CLIENT_CONFIG};
use rooch_key::keystore::AccountKeystore;
use rooch_rpc_api::jsonrpc_types::{ExecuteTransactionResponseView, KeptVMStatusView};
use rooch_types::address::{EthereumAddress, RoochAddress};
use rooch_types::coin_type::CoinID;
use rooch_types::crypto::{RoochKeyPair, Signature};
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::transaction::ethereum::{EthereumTransaction, EthereumTransactionData};
use rooch_types::transaction::{
    authenticator::Authenticator,
    rooch::{RoochTransaction, RoochTransactionData},
};
use rooch_types::H160;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub struct WalletContext<K: Ord, V> {
    client: Arc<RwLock<Option<Client>>>,
    pub config: PersistedConfig<ClientConfig<K, V>>,
}

impl WalletContext<RoochAddress, RoochKeyPair> {
    pub async fn new(config_path: Option<PathBuf>) -> Result<Self, anyhow::Error> {
        let config_dir = config_path.unwrap_or(rooch_config_dir()?);
        let config_path = config_dir.join(ROOCH_CLIENT_CONFIG);
        let config: ClientConfig<RoochAddress, RoochKeyPair> = PersistedConfig::read(&config_path).map_err(|err| {
            anyhow!(
                "Cannot open wallet config file at {:?}. Err: {err}, Use `rooch init` to configuration",
                config_path
            )
        })?;

        let config = config.persisted(&config_path);
        Ok(Self {
            client: Default::default(),
            config,
        })
    }

    pub fn parse_account_arg(&self, arg: String) -> Result<AccountAddress, RoochError> {
        self.parse(arg)
    }

    pub fn parse_account_args(
        &self,
        args: BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, AccountAddress>, RoochError> {
        Ok(args
            .into_iter()
            .map(|(key, value)| (key, self.parse(value).unwrap()))
            .collect())
    }

    pub async fn get_client(&self) -> Result<Client, anyhow::Error> {
        // TODO: Check version

        let read = self.client.read().await;

        Ok(if let Some(client) = read.as_ref() {
            client.clone()
        } else {
            drop(read);
            let client = self
                .config
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
    ) -> RoochResult<RoochTransactionData> {
        let client = self.get_client().await?;
        let chain_id = self.config.get_active_env()?.chain_id;
        let sequence_number = client
            .transaction_count(sender.0, Some(BlockNumber::Latest))
            .await
            .map_err(RoochError::from)?;
        log::debug!("use sequence_number: {}", sequence_number);
        //TODO max gas amount from cli option or dry run estimate
        let tx_data = RoochTransactionData::new(
            sender,
            sequence_number,
            chain_id,
            GasConfig::DEFAULT_MAX_GAS_AMOUNT,
            action,
        );
        Ok(tx_data)
    }

    pub async fn sign(
        &self,
        sender: RoochAddress,
        action: MoveAction,
        coin_id: CoinID,
    ) -> RoochResult<RoochTransaction> {
        let kp = self
            .config
            .keystore
            .get_key_pair_by_coin_id(&sender, coin_id)
            .ok()
            .ok_or_else(|| {
                RoochError::SignMessageError(format!("Cannot find key for address: [{sender}]"))
            })?;

        let tx_data = self.build_tx_data(sender, action).await?;
        let signature = Signature::new_hashed(tx_data.hash().as_bytes(), kp);
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
            .execute_tx(tx)
            .await
            .map_err(|e| RoochError::TransactionError(e.to_string()))
    }

    pub async fn sign_and_execute(
        &self,
        sender: RoochAddress,
        action: MoveAction,
        coin_id: CoinID,
    ) -> RoochResult<ExecuteTransactionResponseView> {
        let tx = self.sign(sender, action, coin_id).await?;
        self.execute(tx).await
    }

    fn parse(&self, account: String) -> Result<AccountAddress, RoochError> {
        if account.starts_with("0x") {
            AccountAddress::from_hex_literal(&account).map_err(|err| {
                RoochError::CommandArgumentError(format!("Failed to parse AccountAddress {}", err))
            })
        } else if let Ok(account_address) = AccountAddress::from_str(&account) {
            Ok(account_address)
        } else {
            let address = match account.as_str() {
                "default" => AccountAddress::from(self.config.active_address.unwrap()),
                _ => Err(RoochError::CommandArgumentError(
                    "Use rooch init configuration".to_owned(),
                ))?,
            };

            Ok(address)
        }
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

impl WalletContext<EthereumAddress, Secp256k1RecoverableKeyPair> {
    pub async fn new(config_path: Option<PathBuf>) -> Result<Self, anyhow::Error> {
        // TODO change to ethereum config dir?
        let config_dir = config_path.unwrap_or(rooch_config_dir()?);
        let config_path = config_dir.join(ROOCH_CLIENT_CONFIG);
        let config: ClientConfig<EthereumAddress, Secp256k1RecoverableKeyPair> = PersistedConfig::read(&config_path).map_err(|err| {
            anyhow!(
                "Cannot open wallet config file at {:?}. Err: {err}, Use `rooch init` to configuration",
                config_path
            )
        })?;

        let config = config.persisted(&config_path);
        Ok(Self {
            client: Default::default(),
            config,
        })
    }

    pub fn parse_account_arg(&self, arg: String) -> Result<H160, RoochError> {
        self.parse(arg)
    }

    pub fn parse_account_args(
        &self,
        args: BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, H160>, RoochError> {
        Ok(args
            .into_iter()
            .map(|(key, value)| (key, self.parse(value).unwrap()))
            .collect())
    }

    pub async fn get_client(&self) -> Result<Client, anyhow::Error> {
        // TODO: Check version

        let read = self.client.read().await;

        Ok(if let Some(client) = read.as_ref() {
            client.clone()
        } else {
            drop(read);
            let client = self
                .config
                .get_active_env()?
                .create_rpc_client(Duration::from_secs(DEFAULT_EXPIRATION_SECS), None)
                .await?;

            self.client.write().await.insert(client).clone()
        })
    }

    pub async fn build_tx_data(
        &self,
        sender: EthereumAddress,
        action: MoveAction,
    ) -> RoochResult<EthereumTransactionData> {
        let client = self.get_client().await?;
        let chain_id = self.config.get_active_env()?.chain_id;
        let nonce = client
            .transaction_count(sender.0, Some(BlockNumber::Latest))
            .await
            .map_err(RoochError::from)?;
        log::debug!("use nonce: {}", nonce);

        // Create a new Transaction instance by providing values for its fields
        let transaction = Transaction {
            hash: H256::zero(),
            nonce,
            block_hash: None,
            block_number: None,
            transaction_index: None,
            from: sender.0,
            to: None,
            value: U256::zero(),
            gas_price: None,
            gas: GasConfig::DEFAULT_MAX_GAS_AMOUNT.into(),
            input: Bytes::from(action.encode()?),
            v: U64::zero(),
            r: U256::zero(),
            s: U256::zero(),
            transaction_type: None,
            access_list: None,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            chain_id: Some(chain_id.into()),
            other: OtherFields::default(),
        };

        // Wrap the created Transaction in an EthereumTransactionData
        let tx_data = EthereumTransactionData(transaction);

        Ok(tx_data)
    }

    pub async fn sign(
        &self,
        sender: EthereumAddress,
        action: MoveAction,
        coin_id: CoinID,
    ) -> RoochResult<EthereumTransaction> {
        let kp = self
            .config
            .keystore
            .get_key_pair_by_coin_id(&sender, coin_id)
            .ok()
            .ok_or_else(|| {
                RoochError::SignMessageError(format!("Cannot find key for address: [{sender}]"))
            })?;

        let tx_data = self.build_tx_data(sender, action).await?;
        let signature = kp.sign_recoverable_with_hash::<Keccak256>(tx_data.0.hash().as_bytes());
        Ok(EthereumTransaction::new(
            tx_data,
            Authenticator::ethereum(signature),
        ))
    }

    pub async fn execute(&self, tx: EthereumTransactionData) -> RoochResult<TransactionReceipt> {
        let client = self.get_client().await?;
        let tx = client
            .send_raw_transaction(tx.0.rlp())
            .await
            .map_err(|e| RoochError::TransactionError(e.to_string()))?;
        let tx_receipt = client
            .transaction_receipt(tx)
            .await
            .map_err(|e| RoochError::TransactionError(e.to_string()))?;
        Ok(tx_receipt)
    }

    pub async fn sign_and_execute(
        &self,
        sender: EthereumAddress,
        action: MoveAction,
        coin_id: CoinID,
    ) -> RoochResult<TransactionReceipt> {
        let tx = self.sign(sender, action, coin_id).await?;
        self.execute(tx).await
    }

    fn parse(&self, account: String) -> Result<H160, RoochError> {
        if account.starts_with("0x") {
            let stripped_account = account.strip_prefix("0x").unwrap_or(&account);
            H160::from_str(stripped_account).map_err(|err| {
                RoochError::CommandArgumentError(format!("Failed to parse AccountAddress {}", err))
            })
        } else if let Ok(account_address) = H160::from_str(&account) {
            Ok(account_address)
        } else {
            let address = match account.as_str() {
                "default" => H160::from(self.config.active_address.unwrap()),
                _ => Err(RoochError::CommandArgumentError(
                    "Use rooch init configuration".to_owned(),
                ))?,
            };

            Ok(address)
        }
    }

    pub fn assert_execute_success(
        &self,
        result: TransactionReceipt,
    ) -> RoochResult<TransactionReceipt> {
        if U64::one() != result.status {
            Err(RoochError::TransactionError(format!(
                "Transaction execution failed: {:?}",
                result.status
            )))
        } else {
            Ok(result)
        }
    }
}
