// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Error;
use clap::Parser;
use moveos_store::config_store::ConfigStore;
use moveos_store::transaction_store::TransactionStore as TxExecutionInfoStore;
use moveos_types::startup_info;
use rooch_config::R_OPT_NET_HELP;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::rooch_network::RoochChainID;
use rooch_types::sequencer::SequencerInfo;
use std::path::PathBuf;

use crate::cli_types::WalletContextOptions;
use crate::commands::db::commands::init;

/// Rollback the state to a specific transaction order.
#[derive(Debug, Parser)]
pub struct RollbackCommand {
    #[clap(long, short = 'o')]
    /// tx order which the state will rollback to
    pub tx_order: u64,

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

impl RollbackCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let tx_order = self.tx_order;
        let (_root, rooch_db, _start_time) = init(self.base_data_dir, self.chain_id);

        let last_sequencer_info = rooch_db
            .rooch_store
            .get_meta_store()
            .get_sequencer_info()?
            .ok_or_else(|| anyhow::anyhow!("Load sequencer info failed"))?;
        let (last_order, last_accumulator_info) = (
            last_sequencer_info.last_order,
            last_sequencer_info.last_accumulator_info.clone(),
        );
        println!("Load latest sequencer order {:?}", last_order);
        println!(
            "Load latest sequencer accumulator info {:?}",
            last_accumulator_info
        );

        if tx_order >= last_order {
            return Err(RoochError::from(Error::msg(format!(
                "tx order {} is greater or equals than last order {}",
                tx_order, last_order
            ))));
        }

        let tx_hashes = rooch_db
            .rooch_store
            .transaction_store
            .get_tx_hashes(vec![tx_order])?;
        // check tx hash exist via tx_order
        if tx_hashes.is_empty() || tx_hashes[0].is_none() {
            return Err(RoochError::from(Error::msg(format!(
                "tx hash not exist via tx order {}",
                tx_order
            ))));
        }
        let tx_hash = tx_hashes[0].unwrap();

        let ledger_tx_opt = rooch_db
            .rooch_store
            .transaction_store
            .get_transaction_by_hash(tx_hash)?;
        if ledger_tx_opt.is_none() {
            println!("the ledger tx not exist via tx_hash {}", tx_hash);
            return Ok(());
        }
        let sequencer_info = ledger_tx_opt.unwrap().sequence_info;
        assert_eq!(sequencer_info.tx_order, tx_order);
        let tx_order = sequencer_info.tx_order;

        let execution_info = rooch_db
            .moveos_store
            .transaction_store
            .get_tx_execution_info(tx_hash)?;

        if execution_info.is_none() {
            return Err(RoochError::from(Error::msg(format!(
                "the tx execution info not exist via tx_hash {}",
                tx_hash
            ))));
        }

        let execution_info = execution_info.unwrap();
        let start_order = tx_order + 1;
        for order in (start_order..=last_order).rev() {
            let tx_hashes = rooch_db
                .rooch_store
                .transaction_store
                .get_tx_hashes(vec![order])?;
            if tx_hashes.is_empty() || tx_hashes[0].is_none() {
                return Err(RoochError::from(Error::msg(format!(
                    "tx hash not exist via tx order {}",
                    order
                ))));
            }
            let tx_hash = tx_hashes[0].unwrap();
            rooch_db.do_revert_tx_ignore_check(tx_hash)?;
        }

        let rollback_sequencer_info = SequencerInfo {
            last_order: tx_order,
            last_accumulator_info: sequencer_info.tx_accumulator_info(),
        };
        rooch_db
            .rooch_store
            .meta_store
            .save_sequencer_info_ignore_check(rollback_sequencer_info)?;
        let startup_info =
            startup_info::StartupInfo::new(execution_info.state_root, execution_info.size);

        rooch_db.moveos_store.save_startup_info(startup_info)?;

        println!(
            "rollback succ, tx_hash: {:?}, tx_order {}, state_root: {:?}",
            tx_hash, tx_order, execution_info.state_root
        );
        Ok(())
    }
}
