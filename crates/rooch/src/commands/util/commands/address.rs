// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use rooch_types::{address::ParsedAddress, error::RoochResult};
use serde::{Deserialize, Serialize};

/// Tool for convert address format
#[derive(Debug, Parser)]
pub struct AddressCommand {
    /// Address to convert, any format which rooch supports
    addr: ParsedAddress,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressOutput {
    pub rooch_address: String,
    pub hex_address: String,
    pub bitcoin_main_address: String,
    pub bitcoin_test_address: String,
    pub bitcoin_regtest_address: String,
    pub bitcoin_segtest_address: String,
}

#[async_trait]
impl CommandAction<AddressOutput> for AddressCommand {
    async fn execute(self) -> RoochResult<AddressOutput> {
        let context = self.context_options.build()?;
        let rooch_addr = context.resolve_rooch_address(self.addr.clone())?;
        let bitcoin_addr = context.resolve_bitcoin_address(self.addr).await?;

        Ok(AddressOutput {
            rooch_address: rooch_addr.to_string(),
            hex_address: rooch_addr.to_hex_literal(),
            bitcoin_main_address: bitcoin_addr.format(bitcoin::Network::Bitcoin)?.to_string(),
            bitcoin_test_address: bitcoin_addr.format(bitcoin::Network::Testnet)?.to_string(),
            bitcoin_regtest_address: bitcoin_addr.format(bitcoin::Network::Regtest)?.to_string(),
            bitcoin_segtest_address: bitcoin_addr.format(bitcoin::Network::Signet)?.to_string(),
        })
    }
}
