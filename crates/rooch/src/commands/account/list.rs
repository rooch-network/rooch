// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// #![allow(unused_imports)]

use anyhow::{Ok, Result};
use clap::Parser;
use moveos_client::Client;
use std::fmt::Debug;

use crate::config::{PersistedConfig, RoochConfig};
use rooch_key::keystore::AccountKeystore;

/// Create a new account on-chain
///
/// An account can be created by transferring coins, or by making an explicit
/// call to create an account.  This will create an account with no coins, and
/// any coins will have to transferred afterwards.
#[derive(Debug, Parser)]
pub struct ListCommand {
    /// RPC client options.
    #[clap(flatten)]
    client: Client,
}

impl ListCommand {
    pub async fn execute(self, config: &mut PersistedConfig<RoochConfig>) -> Result<()> {
        let addresses = config.keystore.addresses();
        let active_address = config.active_address;

        println!("Showing {} results.", addresses.len());
        for address in addresses {
            if active_address == Some(address) {
                println!("{} <= active", &address);
            } else {
                println!("{}", address);
            }
        }

        Ok(())
    }
}
