// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::{derive_builtin_genesis_namespace, open_rooch_db};
use clap::Parser;
use rooch_anomalies::load_tx_anomalies;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::{BuiltinChainID, RoochChainID};
use std::path::PathBuf;

/// Repair the database offline.
/// Help to reach consistency of the database.
#[derive(Debug, Parser)]
pub struct RepairCommand {
    #[clap(
        long,
        help = "perform a thorough and detailed check, which may take more time"
    )]
    pub thorough: bool,
    #[clap(
        long = "exec",
        help = "execute repair, otherwise only report issues. default is false"
    )]
    pub exec: bool,
    #[clap(
        long = "fast-fail",
        help = "fail fast on the first error, otherwise continue to check all issues"
    )]
    pub fast_fail: bool,
    #[clap(long = "sync-mode", help = "if true, no DA block will be generated")]
    pub sync_mode: bool,

    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: BuiltinChainID,
}

impl RepairCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let (_root, rooch_db, _start_time) = open_rooch_db(
            self.base_data_dir,
            Some(RoochChainID::Builtin(self.chain_id)),
        );

        let genesis_namespace = derive_builtin_genesis_namespace(self.chain_id)?;
        let tx_anomalies = load_tx_anomalies(genesis_namespace.clone())?;

        let (issues, fixed) = rooch_db.repair(
            self.thorough,
            self.exec,
            self.fast_fail,
            self.sync_mode,
            tx_anomalies,
        )?;

        println!("issues found: {}, fixed: {}", issues, fixed);

        Ok(())
    }
}
