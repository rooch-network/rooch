// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use crate::utils::get_sequencer_keypair;
use async_trait::async_trait;
use moveos_types::h256::H256;
use rooch_sequencer::actor::sequencer::sign_tx_order;
use rooch_types::error::RoochResult;

/// Get transactions by hashes
#[derive(Debug, clap::Parser)]
pub struct SignOrderCommand {
    /// Transaction's hash
    #[clap(long)]
    pub tx_hash: H256,
    #[clap(long)]
    pub tx_order: u64,
    #[clap(long)]
    pub sequencer_account: Option<String>,
    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<Vec<u8>> for SignOrderCommand {
    async fn execute(self) -> RoochResult<Vec<u8>> {
        let sequencer_keypair =
            get_sequencer_keypair(self.context_options, self.sequencer_account)?;
        let tx_order_sign = sign_tx_order(self.tx_order, self.tx_hash, &sequencer_keypair);
        Ok(tx_order_sign)
    }
}
