// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use crate::commands::db::commands::init;
use anyhow::Error;
use clap::Parser;
use moveos_common::utils::to_bytes;
use moveos_store::config_store::STARTUP_INFO_KEY;
use moveos_store::transaction_store::TransactionStore as TxExecutionInfoStore;
use moveos_store::CONFIG_STARTUP_INFO_COLUMN_FAMILY_NAME;
use moveos_types::startup_info;
use raw_store::rocks::batch::WriteBatch;
use raw_store::traits::DBStore;
use rooch_config::R_OPT_NET_HELP;
use rooch_store::meta_store::SEQUENCER_INFO_KEY;
use rooch_store::META_SEQUENCER_INFO_COLUMN_FAMILY_NAME;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::rooch_network::RoochChainID;
use rooch_types::sequencer::SequencerInfo;
use std::path::PathBuf;

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
        if tx_order == 0 {
            return Err(RoochError::from(Error::msg(
                "tx order should be greater than 0",
            )));
        }
        let (_root, rooch_db, _start_time) = init(self.base_data_dir, self.chain_id);

        // check
        // 1. tx_hash exist via tx_order
        let tx_hashes = rooch_db
            .rooch_store
            .transaction_store
            .get_tx_hashes(vec![tx_order])?;
        if tx_hashes.is_empty() || tx_hashes[0].is_none() {
            return Err(RoochError::from(Error::msg(format!(
                "rollback tx failed: tx_hash not found for tx_order {:?}",
                tx_order
            ))));
        }
        let tx_hash = tx_hashes[0].unwrap();
        // 2. tx_order must be less than last_order
        let last_sequencer_info = rooch_db
            .rooch_store
            .get_meta_store()
            .get_sequencer_info()?
            .ok_or_else(|| anyhow::anyhow!("Load sequencer info failed"))?;
        let last_order = last_sequencer_info.last_order;
        if tx_order >= last_order {
            return Err(RoochError::from(Error::msg(format!(
                "rollback tx failed: tx_order {} should be less than last_order {}",
                tx_order, last_order
            ))));
        }
        // 3. tx saved, sequenced, executed
        // 3.1 tx saved
        let ledger_tx_opt = rooch_db
            .rooch_store
            .transaction_store
            .get_transaction_by_hash(tx_hash)?;
        if ledger_tx_opt.is_none() {
            return Err(RoochError::from(Error::msg(format!(
                "rollback tx failed: tx not exist via tx_hash {}",
                tx_hash
            ))));
        }
        // 3.2 tx sequenced
        let sequencer_info = ledger_tx_opt.unwrap().sequence_info;
        assert_eq!(sequencer_info.tx_order, tx_order);
        // 3.3 tx executed
        let execution_info = rooch_db
            .moveos_store
            .transaction_store
            .get_tx_execution_info(tx_hash)?;
        if execution_info.is_none() {
            return Err(RoochError::from(Error::msg(format!(
                "rollback tx failed: tx not executed via tx_hash {}",
                tx_hash
            ))));
        }
        let execution_info = execution_info.unwrap();

        // rollback
        let start_order = tx_order + 1;
        for tx_order_i in (start_order..=last_order).rev() {
            let tx_hashes = rooch_db
                .rooch_store
                .transaction_store
                .get_tx_hashes(vec![tx_order_i])?;
            if tx_hashes.is_empty() || tx_hashes[0].is_none() {
                println!(
                    "rollback tx error: tx_hash not found for tx_order {:?}: may have been revert/database inconsistent",
                    tx_order_i
                );
                // tx_hash lost:
                // 1. rollback incomplete cause last_order not updated
                // 2. database is inconsistent (use another method to check/repair)
                //
                // it's okay to continue rollback, after reverting all txs; the last_order will be updated later
                continue;
            }
            let tx_hash = tx_hashes[0].unwrap();
            rooch_db.revert_tx_unsafe(tx_order_i, tx_hash)?;
        }

        let rollback_sequencer_info = SequencerInfo {
            last_order: tx_order,
            last_accumulator_info: sequencer_info.tx_accumulator_info(),
        };
        let rollback_startup_info =
            startup_info::StartupInfo::new(execution_info.state_root, execution_info.size);

        let inner_store = &rooch_db.rooch_store.store_instance;
        let mut write_batch = WriteBatch::new();
        let cf_names = vec![
            META_SEQUENCER_INFO_COLUMN_FAMILY_NAME,
            CONFIG_STARTUP_INFO_COLUMN_FAMILY_NAME,
        ];
        // save sequencer info and startup info for setup with rollback tx values
        write_batch.put(
            to_bytes(SEQUENCER_INFO_KEY).unwrap(),
            to_bytes(&rollback_sequencer_info).unwrap(),
        )?;
        write_batch.put(
            to_bytes(STARTUP_INFO_KEY).unwrap(),
            to_bytes(&rollback_startup_info).unwrap(),
        )?;

        inner_store.write_batch_across_cfs(cf_names, write_batch, true)?;

        println!(
            "rollback tx succeed, tx_hash: {:?}, tx_order {}, state_root: {:?}",
            tx_hash, tx_order, execution_info.state_root
        );
        Ok(())
    }
}
