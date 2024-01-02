// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{TransactionOptions, WalletContextOptions};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use clap::Parser;
use rooch_types::{
    address::{BitcoinAddress, MultiChainAddress, RoochAddress},
    error::{RoochError, RoochResult},
    multichain_id::RoochMultiChainID,
};

/// Binding a
#[derive(Debug, Parser)]
pub struct BindingCommand {
    /// L1 chain id    
    #[clap(long, short = 'c')]
    pub l1_chain: RoochMultiChainID,

    /// L1 chain address
    #[clap(long)]
    pub l1_address: String,

    #[clap(
        long,
        env = "BITCOIN_RPC_URL",
        requires = "btc-rpc-username",
        requires = "btc-rpc-password"
    )]
    pub btc_rpc_url: String,

    #[clap(long, id = "btc-rpc-username", env = "BTC_RPC_USERNAME")]
    pub btc_rpc_username: String,

    #[clap(long, id = "btc-rpc-password", env = "BTC_RPC_PASSWORD")]
    pub btc_rpc_password: String,

    #[clap(flatten)]
    pub tx_options: TransactionOptions,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

impl BindingCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let multichain_address = MultiChainAddress::try_from_str_with_multichain_id(
            self.l1_chain,
            &self.l1_address.as_str(),
        )?;
        let wallet_context = self.context_options.build()?;
        let sender = wallet_context.resolve_address(self.tx_options.sender)?;
        match self.l1_chain {
            RoochMultiChainID::Bitcoin => {
                Self::binding_bitcoin_adddress(
                    self.btc_rpc_url,
                    self.btc_rpc_username,
                    self.btc_rpc_password,
                    sender.into(),
                    multichain_address.try_into()?,
                )
                .await?;
                Ok(())
            }
            RoochMultiChainID::Rooch => Err(RoochError::CommandArgumentError(
                "Can not binding Rooch address".to_owned(),
            )),
            _ => Err(RoochError::CommandArgumentError(
                "Does not support this chain".to_owned(),
            )),
        }
    }

    async fn binding_bitcoin_adddress(
        btc_rpc_url: String,
        btc_rpc_user_name: String,
        btc_rpc_password: String,
        _sender: RoochAddress,
        bitcoin_address: BitcoinAddress,
    ) -> anyhow::Result<String> {
        let rpc = Client::new(
            btc_rpc_url.as_str(),
            Auth::UserPass(btc_rpc_user_name, btc_rpc_password),
        )?;
        let chain_info = rpc.get_blockchain_info()?;
        let bitcoin_chain_network =
            rooch_types::bitcoin::network::Network::try_from(chain_info.chain)?;
        let bitcoin_address_str = bitcoin_address.format(bitcoin_chain_network.to_num())?;
        let message = "rooch";
        let result = rpc
            .call::<String>(
                "signmessage",
                &[
                    serde_json::to_value(bitcoin_address_str)?,
                    serde_json::to_value(message)?,
                ],
            )
            .map_err(|e| RoochError::UnexpectedError(e.to_string()))?;
        Ok(result)
    }
}
