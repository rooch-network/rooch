// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::da::commands::LedgerTxGetter;
use anyhow::anyhow;
use heed::byteorder::BigEndian;
use heed::types::{SerdeBincode, U64};
use heed::{Database, Env, EnvOpenOptions};
use moveos_types::h256::H256;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::path::PathBuf;

const MAP_SIZE: usize = 1 << 34; // 16G
const MAX_DBS: u32 = 16;
const ORDER_DATABASE_NAME: &str = "order_db";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TxPosition {
    pub tx_hash: H256,
    pub block_number: u128,
}

/// Index tx_order:tx_hash:block_number
#[derive(Debug, clap::Parser)]
pub struct IndexCommand {
    #[clap(long = "segment-dir", short = 's')]
    pub segment_dir: PathBuf,
    #[clap(long = "index", short = 'i')]
    pub index_path: PathBuf,
    #[clap(
        long = "reset-from",
        help = "Reset from tx order(inclusive), all tx orders after this will be re-indexed"
    )]
    pub reset_from: Option<u64>,
    #[clap(long = "max-block-number", help = "Max block number to index")]
    pub max_block_number: Option<u128>,
}

pub struct Indexer {
    db_env: Env,
    last_tx_order: u64,
    last_block_number: u128,
}

impl Indexer {
    pub fn new(db_path: PathBuf, reset_from: Option<u64>) -> anyhow::Result<Self> {
        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(MAP_SIZE) // 16G
                .max_dbs(MAX_DBS)
                .open(db_path)?
        };
        let mut indexer = Indexer {
            db_env: env,
            last_tx_order: 0,
            last_block_number: 0,
        };
        if let Some(from) = reset_from {
            indexer.reset_from(from)?;
        }

        indexer.init_cursor()?;
        Ok(indexer)
    }

    fn init_cursor(&mut self) -> anyhow::Result<()> {
        // init cursor by search last tx_order
        let rtxn = self.db_env.read_txn()?;
        let db: Database<U64<BigEndian>, SerdeBincode<TxPosition>> = self
            .db_env
            .open_database(&rtxn, Some(ORDER_DATABASE_NAME))?
            .ok_or(anyhow::anyhow!("db not found"))?;
        if let Some((k, v)) = db.last(&rtxn)? {
            self.last_tx_order = k;
            self.last_block_number = v.block_number;
        }
        rtxn.commit()?;
        Ok(())
    }

    fn reset_from(&self, from: u64) -> anyhow::Result<()> {
        let mut wtxn = self.db_env.write_txn()?;
        let db: Database<U64<BigEndian>, SerdeBincode<TxPosition>> = self
            .db_env
            .create_database(&mut wtxn, Some(ORDER_DATABASE_NAME))?;

        let range = from..;
        db.delete_range(&mut wtxn, &range)?;
        wtxn.commit()?;
        Ok(())
    }
}

impl IndexCommand {
    pub fn execute(self) -> anyhow::Result<()> {
        let db_path = self.index_path.clone();
        let reset_from = self.reset_from;
        let indexer = Indexer::new(db_path, reset_from)?;

        let ledger_tx_loader = LedgerTxGetter::new(self.segment_dir)?;
        let mut block_number = indexer.last_block_number; // avoiding partial indexing
        let mut expected_tx_order = indexer.last_tx_order + 1;
        let stop_at = if let Some(max_block_number) = self.max_block_number {
            max(max_block_number, ledger_tx_loader.get_max_chunk_id())
        } else {
            ledger_tx_loader.get_max_chunk_id()
        };

        let mut wtxn = indexer.db_env.write_txn()?;
        let db: Database<U64<BigEndian>, SerdeBincode<TxPosition>> = indexer
            .db_env
            .create_database(&mut wtxn, Some(ORDER_DATABASE_NAME))?;

        loop {
            if block_number > stop_at {
                break;
            }
            let tx_list = ledger_tx_loader.load_ledger_tx_list(block_number, true)?;
            let tx_list = tx_list.unwrap();
            for mut ledger_tx in tx_list {
                let tx_order = ledger_tx.sequence_info.tx_order;
                if tx_order < expected_tx_order {
                    continue;
                }
                if tx_order != expected_tx_order {
                    return Err(anyhow!(
                        "tx_order not continuous, expect: {}, got: {}",
                        expected_tx_order,
                        tx_order
                    ));
                }
                let tx_hash = ledger_tx.tx_hash();
                let tx_position = TxPosition {
                    tx_hash,
                    block_number,
                };
                db.put(&mut wtxn, &tx_order, &tx_position)?;
                expected_tx_order += 1;
            }
            block_number += 1;
        }
        wtxn.commit()?;
        indexer.db_env.force_sync()?;
        Ok(())
    }
}
