// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::Error;
use clap::Parser;
use metrics::RegistryService;

use moveos_store::transaction_store::TransactionStore as TxExecutionInfoStore;
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::object::ObjectMeta;
use rooch_config::{RoochOpt, R_OPT_NET_HELP};
use rooch_db::RoochDB;
use rooch_genesis::RoochGenesis;
use rooch_store::transaction_store::TransactionStore;
use rooch_store::RoochStore;
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::rooch_network::RoochChainID;
use rooch_types::sequencer::SequencerInfo;

use crate::cli_types::WalletContextOptions;

/// Revert tx by db command.
#[derive(Debug, Parser)]
pub struct RevertTxCommand {
    #[clap(long, short = 'o')]
    /// tx order
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

impl RevertTxCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let tx_order = self.tx_order;
        let (_root, moveos_store, rooch_store, _start_time) = self.init();

        let last_sequencer_info = rooch_store
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

        let tx_hashes = rooch_store
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

        let ledger_tx_opt = rooch_store
            .transaction_store
            .get_transaction_by_hash(tx_hash)?;
        if ledger_tx_opt.is_none() {
            println!("the ledger tx not exist via tx_hash {}", tx_hash);
            return Ok(());
        }
        let sequencer_info = ledger_tx_opt.unwrap().sequence_info;
        let tx_order = sequencer_info.tx_order;
        // check last order equals to sequencer tx order via tx_hash
        if sequencer_info.tx_order != last_sequencer_info.last_order {
            return Err(RoochError::from(Error::msg(format!(
                "the last order {} not match current sequencer info tx order {}, tx_hash {}",
                last_sequencer_info.last_order, sequencer_info.tx_order, tx_hash
            ))));
        }

        // check only write tx sequence info succ, but not write tx execution info
        let execution_info = moveos_store
            .transaction_store
            .get_tx_execution_info(tx_hash)?;
        if execution_info.is_some() {
            return Err(RoochError::from(Error::msg(format!(
                "the tx execution info already exist via tx_hash {}, revert tx failed",
                tx_hash
            ))));
        }

        let previous_tx_order = last_order - 1;
        let previous_tx_hash_opt = rooch_store
            .transaction_store
            .get_tx_hashes(vec![previous_tx_order])?;
        if previous_tx_hash_opt.is_empty() || previous_tx_hash_opt[0].is_none() {
            return Err(RoochError::from(Error::msg(format!(
                "the previous tx hash not exist via previous_tx_order  {}",
                previous_tx_order
            ))));
        }
        let previous_tx_hash = previous_tx_hash_opt[0].unwrap();
        let previous_ledger_tx_opt = rooch_store
            .transaction_store
            .get_transaction_by_hash(previous_tx_hash)?;
        if previous_ledger_tx_opt.is_none() {
            return Err(RoochError::from(Error::msg(format!(
                "the previous ledger tx not exist via tx_hash {}, revert tx failed",
                previous_tx_hash
            ))));
        }
        let previous_sequencer_info = previous_ledger_tx_opt.unwrap().sequence_info;

        let revert_sequencer_info = SequencerInfo::new(
            previous_tx_order,
            previous_sequencer_info.tx_accumulator_info(),
        );
        rooch_store
            .meta_store
            .save_sequencer_info_ignore_check(revert_sequencer_info)?;
        rooch_store.remove_transaction(tx_hash, tx_order)?;

        println!(
            "revert tx succ, tx_hash: {:?}, tx_order {}",
            tx_hash, tx_order
        );
        Ok(())
    }

    fn init(self) -> (ObjectMeta, MoveOSStore, RoochStore, SystemTime) {
        let start_time = SystemTime::now();

        let opt = RoochOpt::new_with_default(self.base_data_dir, self.chain_id, None).unwrap();
        let registry_service = RegistryService::default();
        let rooch_db =
            RoochDB::init(opt.store_config(), &registry_service.default_registry()).unwrap();
        let genesis = RoochGenesis::load_or_init(opt.network(), &rooch_db).unwrap();
        let root = genesis.genesis_root().clone();
        (
            root,
            rooch_db.moveos_store,
            rooch_db.rooch_store,
            start_time,
        )
    }
}
