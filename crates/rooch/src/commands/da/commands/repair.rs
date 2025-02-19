// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::WalletContextOptions;
use crate::commands::da::commands::{write_down_segments, LedgerTxGetter};
use crate::utils::{get_sequencer_keypair, open_inner_rocks, open_rooch_db};
use accumulator::accumulator_info::AccumulatorInfo;
use accumulator::tree_store::rocks::RocksAccumulatorStore;
use accumulator::{Accumulator, MerkleAccumulator};
use clap::Parser;
use moveos_types::h256::H256;
use rooch_config::R_OPT_NET_HELP;
use rooch_store::{RoochStore, TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME};
use rooch_types::crypto::RoochKeyPair;
use rooch_types::rooch_network::RoochChainID;
use rooch_types::transaction::{LedgerTransaction, TransactionSequenceInfo};
use std::path::PathBuf;
use std::sync::Arc;

/// Repair certain segments with tx_order issues.
/// Prerequisite:
/// 1. Max sequenced tx in DB >= first tx_order in the segment.
///
/// Note: This command won't persist any changes to the RoochStore.
#[derive(Debug, Parser)]
pub struct RepairCommand {
    #[clap(long = "segment-dir")]
    pub segment_dir: PathBuf,
    #[clap(long = "output")]
    pub output: PathBuf,
    #[clap(long = "chunk-id")]
    pub chunk_id: u128,
    #[clap(long)]
    pub sequencer_account: Option<String>,
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
    #[clap(long = "db-path", help = "Path to the Accumulator DB directory")]
    pub db_path: String,
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,
}

impl RepairCommand {
    pub async fn execute(self) -> anyhow::Result<()> {
        let sequencer_keypair =
            get_sequencer_keypair(self.context_options, self.sequencer_account)?;
        let segment_dir = self.segment_dir;
        let ledger_tx_loader = LedgerTxGetter::new(segment_dir)?;
        let (_root, rooch_db, _start_time) = open_rooch_db(self.base_data_dir, self.chain_id);
        let rooch_store = rooch_db.rooch_store.clone();
        let tx_list = ledger_tx_loader
            .load_ledger_tx_list(self.chunk_id, true, false)
            .await?
            .unwrap();
        let mut repair = InnerRepair::new(
            self.chunk_id,
            tx_list,
            rooch_store,
            sequencer_keypair,
            self.db_path,
            self.output,
        )?;
        repair.run()?;
        Ok(())
    }
}

struct InnerRepair {
    chunk_id: u128,
    sequencer_keypair: RoochKeyPair,
    tx_list: Vec<LedgerTransaction>,
    first_tx_order: u64,
    last_tx_order: u64,
    min_timestamp: u64,
    tx_accumulator: MerkleAccumulator,
    output: PathBuf,
}

impl InnerRepair {
    fn new(
        chunk_id: u128,
        tx_list: Vec<LedgerTransaction>,
        rooch_store: RoochStore,
        sequencer_keypair: RoochKeyPair,
        db_path: String,
        output: PathBuf,
    ) -> anyhow::Result<Self> {
        let first_tx_order = tx_list
            .first()
            .map(|tx| tx.sequence_info.tx_order)
            .expect("tx_list should not be empty");
        let last_tx_order = tx_list
            .last()
            .map(|tx| tx.sequence_info.tx_order)
            .expect("tx_list should not be empty");

        let tx_opt = rooch_store
            .get_transaction_store()
            .get_tx_by_order(first_tx_order - 1)?
            .unwrap_or_else(|| panic!("tx_order: {} not found", first_tx_order - 1));
        let min_timestamp = tx_opt.sequence_info.tx_timestamp;

        let tx_accumulator = Self::build_accumulator(first_tx_order, rooch_store.clone(), db_path)?;

        Ok(Self {
            chunk_id,
            sequencer_keypair,
            tx_list,
            first_tx_order,
            last_tx_order,
            min_timestamp,
            tx_accumulator,
            output,
        })
    }

    fn derive_tx_timestamp(&self, tx_order: u64) -> u64 {
        if tx_order == self.first_tx_order {
            return self.min_timestamp;
        }

        self.get_tx_timestamp(tx_order - 1)
            .expect("tx should exist")
    }

    fn get_tx_timestamp(&self, tx_order: u64) -> Option<u64> {
        self.tx_list
            .iter()
            .find(|tx| tx.sequence_info.tx_order == tx_order)
            .map(|tx| tx.sequence_info.tx_timestamp)
    }

    fn build_accumulator(
        first_tx_order: u64,
        rooch_store: RoochStore,
        db_path: String,
    ) -> anyhow::Result<MerkleAccumulator> {
        let start_order = first_tx_order - 1;
        let mut start_tx = rooch_store
            .transaction_store
            .get_tx_by_order(start_order)?
            .unwrap_or_else(|| panic!("tx_order: {} not found", start_order));
        let start_sequence_info = start_tx.sequence_info.clone();
        if start_sequence_info.tx_order != start_order {
            return Err(anyhow::anyhow!(
                "inconsistent sequence info for tx_order: {}",
                start_order
            ));
        }

        let last_sequencer_info = start_sequence_info;
        let last_accumulator_info = AccumulatorInfo::new(
            last_sequencer_info.tx_accumulator_root,
            last_sequencer_info
                .tx_accumulator_frozen_subtree_roots
                .clone(),
            last_sequencer_info.tx_accumulator_num_leaves,
            last_sequencer_info.tx_accumulator_num_nodes,
        );

        let accumulator_db = open_inner_rocks(
            &db_path,
            vec![TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME.to_string()],
            false,
        )?;
        let tx_accumulator = MerkleAccumulator::new_with_info(
            last_accumulator_info,
            Arc::new(RocksAccumulatorStore::new(
                accumulator_db,
                TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME.to_string(),
            )),
        );
        let proof = tx_accumulator.get_proof(start_order)?.unwrap();
        let start_tx_hash = start_tx.tx_hash();
        proof.verify(tx_accumulator.root_hash(), start_tx_hash, start_order)?;
        Ok(tx_accumulator)
    }

    fn append_to(
        &mut self,
        tx_order: u64,
        tx_hash: H256,
        timestamp: u64,
    ) -> anyhow::Result<TransactionSequenceInfo> {
        let tx_order_signature =
            LedgerTransaction::sign_tx_order(tx_order, tx_hash, &self.sequencer_keypair);
        let _tx_accumulator_root = self.tx_accumulator.append(vec![tx_hash].as_slice())?;

        let tx_accumulator_info = self.tx_accumulator.get_info();
        let sequence_info = TransactionSequenceInfo::new(
            tx_order,
            tx_order_signature,
            tx_accumulator_info,
            timestamp,
        );
        Ok(sequence_info)
    }

    fn verify(&mut self, mut new_tx_list: Vec<LedgerTransaction>) -> anyhow::Result<()> {
        for tx in new_tx_list.iter_mut() {
            let tx_order = tx.sequence_info.tx_order;
            let tx_hash = tx.tx_hash();
            let proof = self.tx_accumulator.get_proof(tx_order)?.unwrap();
            proof.verify(self.tx_accumulator.root_hash(), tx_hash, tx_order)?;
        }

        Ok(())
    }

    fn run(&mut self) -> anyhow::Result<()> {
        let mut expected_tx_order = self.first_tx_order;

        let mut old_tx_list = self.tx_list.clone();
        let mut new_tx_list = Vec::with_capacity(self.tx_list.len());
        for tx in old_tx_list.iter_mut() {
            let tx_order = tx.sequence_info.tx_order;
            let tx_hash = tx.tx_hash();
            if tx_order == expected_tx_order {
                let new_tx_sequence_info =
                    self.append_to(tx_order, tx_hash, tx.sequence_info.tx_timestamp)?;
                assert_eq!(new_tx_sequence_info, tx.sequence_info);
                new_tx_list.push(LedgerTransaction {
                    data: tx.data.clone(),
                    sequence_info: new_tx_sequence_info,
                });
                expected_tx_order += 1;
                continue;
            }

            tracing::info!(
                "repairing tx_order: {}, expected_tx_order: {}",
                tx_order,
                expected_tx_order
            );

            let timestamp = self.derive_tx_timestamp(expected_tx_order);
            let new_tx_sequence_info = self.append_to(expected_tx_order, tx_hash, timestamp)?;
            new_tx_list.push(LedgerTransaction {
                data: tx.data.clone(),
                sequence_info: new_tx_sequence_info,
            });
            expected_tx_order += 1;
        }

        self.verify(new_tx_list.clone())?;

        tracing::info!("verify passed. writing down repaired segments");

        write_down_segments(
            self.chunk_id,
            self.first_tx_order,
            self.last_tx_order,
            &new_tx_list,
            &self.sequencer_keypair,
            self.output.clone(),
        )?;

        Ok(())
    }
}
