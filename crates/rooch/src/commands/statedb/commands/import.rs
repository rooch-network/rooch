// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use anyhow::Result;
use clap::Parser;
use moveos_store::MoveOSStore;
use moveos_types::h256::H256;
use moveos_types::state::{KeyState, State};
use rooch_config::R_OPT_NET_HELP;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use smt::{TreeChangeSet, UpdateSet};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const BATCH_SIZE: usize = 2000;

/// Import statedb
#[derive(Debug, Parser)]
pub struct ImportCommand {
    #[clap(long, short = 'i')]
    /// import input file. like ~/.rooch/local/utxo.csv or utxo.csv
    pub input: PathBuf,

    #[clap(long = "data-dir", short = 'd')]
    /// Path to data dir, this dir is base dir, the final data_dir is base_dir/chain_network_name
    pub base_data_dir: Option<PathBuf>,

    /// If local chainid, start the service with a temporary data store.
    /// All data will be deleted when the service is stopped.
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct UTXOData {
//     /// The txid of the UTXO
//     pub txid: String,
//     /// The vout of the UTXO
//     pub vout: u32,
//     pub value: u64,
//     pub address: String,
//     // pub seals: SimpleMultiMap<MoveString, ObjectID>,
// }
//
// impl UTXOData {
//     pub fn new(txid: String, vout: u32, value: u64, address: String) -> Self {
//         Self {
//             txid,
//             vout,
//             value,
//             address,
//         }
//     }
// }

impl ImportCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let mut _context = self.context_options.build()?;
        // let client = context.get_client().await?;

        todo!()
    }
}

pub fn apply_fields<I>(
    moveos_store: &MoveOSStore,
    pre_state_root: H256,
    update_set: I,
) -> Result<TreeChangeSet>
where
    I: Into<UpdateSet<KeyState, State>>,
{
    let tree_change_set = moveos_store
        .statedb
        .update_fields(pre_state_root, update_set)?;
    Ok(tree_change_set)
}

pub fn apply_nodes(moveos_store: &MoveOSStore, nodes: BTreeMap<H256, Vec<u8>>) -> Result<()> {
    moveos_store.statedb.node_store.write_nodes(nodes)?;
    Ok(())
}
