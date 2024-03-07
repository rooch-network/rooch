// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoincore_rpc::{bitcoin, Auth, Client, RpcApi};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct BitcoinClientConfig {
    pub btc_rpc_url: String,
    pub btc_rpc_user_name: String,
    pub btc_rpc_password: String,
}

impl BitcoinClientConfig {
    pub fn new(btc_rpc_url: String, btc_rpc_user_name: String, btc_rpc_password: String) -> Self {
        Self {
            btc_rpc_url,
            btc_rpc_user_name,
            btc_rpc_password,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BitcoinClient {
    rpc_client: Client,
}

impl BitcoinClient {
    pub fn new(config: BitcoinClientConfig) -> anyhow::Result<Self> {
        let rpc = Client::new(
            config.btc_rpc_url.as_str(),
            Auth::UserPass(config.btc_rpc_user_name, config.btc_rpc_password),
        )?;
        Ok(Self { rpc_client: rpc })
    }

    async fn get_block(&self, hash: &bitcoin::BlockHash) -> anyhow::Result<(bitcoin::Block)> {
        let block = self.rpc_client.get_block(hash)?;

        Ok(block)
    }
}
