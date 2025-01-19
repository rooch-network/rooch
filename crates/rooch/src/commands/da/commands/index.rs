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
use std::io::{BufRead, BufWriter, Write};
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

const MAP_SIZE: usize = 1 << 34; // 16G
const MAX_DBS: u32 = 1;
const ORDER_DATABASE_NAME: &str = "order_db";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TxPosition {
    pub tx_hash: H256,
    pub block_number: u128,
}

/// Index tx_order:tx_hash:block_number
#[derive(Debug, clap::Parser)]
pub struct IndexCommand {
    #[clap(long = "segment-dir", short = 's')]
    pub segment_dir: Option<PathBuf>,
    #[clap(long = "index", short = 'i')]
    pub index_path: PathBuf,
    #[clap(
        long = "reset-from",
        help = "Reset from tx order(inclusive), all tx orders after this will be re-indexed"
    )]
    pub reset_from: Option<u64>,
    #[clap(long = "max-block-number", help = "Max block number to index")]
    pub max_block_number: Option<u128>,
    #[clap(long = "file", help = "Load/dump file-based index")]
    pub index_file_path: Option<PathBuf>,
    #[clap(long = "dump", help = "Dump index to file")]
    pub dump: bool,
}

pub struct Indexer {
    db_env: Env,
    db: Database<U64<BigEndian>, SerdeBincode<TxPosition>>,
    last_tx_order: u64,
    last_block_number: u128,
}

#[derive(Debug, Serialize)]
pub struct IndexerStats {
    pub total_tx_count: u64,
    pub last_tx_order: u64,
    pub last_block_number: u128,
}

impl Indexer {
    pub fn load_or_dump(db_path: PathBuf, file_path: PathBuf, dump: bool) -> anyhow::Result<()> {
        if dump {
            let indexer = Indexer::new(db_path, None)?;
            indexer.dump_to_file(file_path)
        } else {
            Indexer::load_from_file(db_path, file_path)
        }
    }

    pub fn dump_to_file(&self, file_path: PathBuf) -> anyhow::Result<()> {
        let db = self.db;
        let file = std::fs::File::create(file_path)?;
        let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file.try_clone().unwrap());
        let rtxn = self.db_env.read_txn()?;
        let mut iter = db.iter(&rtxn)?;
        while let Some((k, v)) = iter.next().transpose()? {
            writeln!(writer, "{}:{:?}:{}", k, v.tx_hash, v.block_number)?;
        }
        drop(iter);
        rtxn.commit()?;
        writer.flush().expect("Unable to flush writer");
        file.sync_data().expect("Unable to sync file");
        Ok(())
    }

    pub fn load_from_file(db_path: PathBuf, file_path: PathBuf) -> anyhow::Result<()> {
        let mut last_tx_order = 0;
        let mut last_tx_hash = H256::zero();
        let mut last_block_number = 0;

        let db_env = Self::create_env(db_path.clone())?;
        let file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);

        let mut wtxn = db_env.write_txn()?; // Begin write_transaction early for create/put

        let mut is_verify = false;
        let db: Database<U64<BigEndian>, SerdeBincode<TxPosition>> =
            match db_env.open_database(&wtxn, Some(ORDER_DATABASE_NAME)) {
                Ok(Some(db)) => {
                    info!("Database already exists, verify mode");
                    is_verify = true;
                    db
                }
                Ok(None) => db_env.create_database(&mut wtxn, Some(ORDER_DATABASE_NAME))?,
                Err(e) => return Err(e.into()), // Proper error propagation
            };
        wtxn.commit()?;

        let mut wtxn = db_env.write_txn()?;

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() != 3 {
                return Err(anyhow!("invalid line: {}", line));
            }
            let tx_order = parts[0].parse::<u64>()?;
            let tx_hash = H256::from_str(parts[1])?;
            let block_number = parts[2].parse::<u128>()?;
            let tx_position = TxPosition {
                tx_hash,
                block_number,
            };

            if is_verify {
                let rtxn = db_env.read_txn()?;
                let ret = db.get(&rtxn, &tx_order)?;
                let ret = ret.ok_or(anyhow!("tx_order not found: {}", tx_order))?;
                rtxn.commit()?;
                assert_eq!(ret, tx_position);
            } else {
                db.put(&mut wtxn, &tx_order, &tx_position)?;
            }

            last_tx_order = tx_order;
            last_tx_hash = tx_hash;
            last_block_number = block_number;
        }

        wtxn.commit()?;

        if last_tx_order != 0 {
            let rtxn = db_env.read_txn()?;
            let ret = db.last(&rtxn)?;
            assert_eq!(
                ret,
                Some((
                    last_tx_order,
                    TxPosition {
                        tx_hash: last_tx_hash,
                        block_number: last_block_number,
                    }
                ))
            );
        }

        {
            let rtxn = db_env.read_txn()?;
            let final_count = db.iter(&rtxn)?.count();
            info!("Final record count: {}", final_count);
            rtxn.commit()?;
        }

        db_env.force_sync()?;

        Ok(())
    }

    pub fn new(db_path: PathBuf, reset_from: Option<u64>) -> anyhow::Result<Self> {
        let db_env = Self::create_env(db_path)?;
        let mut txn = db_env.write_txn()?;
        let db: Database<U64<BigEndian>, SerdeBincode<TxPosition>> =
            db_env.create_database(&mut txn, Some(ORDER_DATABASE_NAME))?;
        txn.commit()?;

        let mut indexer = Indexer {
            db_env,
            db,
            last_tx_order: 0,
            last_block_number: 0,
        };
        if let Some(from) = reset_from {
            indexer.reset_from(from)?;
        }

        indexer.init_cursor()?;
        Ok(indexer)
    }

    pub fn get_tx_position(&self, tx_order: u64) -> anyhow::Result<Option<TxPosition>> {
        let rtxn = self.db_env.read_txn()?;
        let db = self.db;
        let ret = db.get(&rtxn, &tx_order)?;
        rtxn.commit()?;
        Ok(ret)
    }

    fn create_env(db_path: PathBuf) -> anyhow::Result<Env> {
        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(MAP_SIZE) // 16G
                .max_dbs(MAX_DBS)
                .open(db_path)?
        };
        Ok(env)
    }

    // init cursor by search last tx_order
    fn init_cursor(&mut self) -> anyhow::Result<()> {
        let rtxn = self.db_env.read_txn()?;
        let db = self.db;
        if let Some((k, v)) = db.last(&rtxn)? {
            self.last_tx_order = k;
            self.last_block_number = v.block_number;
        }
        rtxn.commit()?;
        Ok(())
    }

    fn reset_from(&self, from: u64) -> anyhow::Result<()> {
        let mut wtxn = self.db_env.write_txn()?;
        let db = self.db;

        let range = from..;
        let deleted_count = db.delete_range(&mut wtxn, &range)?;
        wtxn.commit()?;
        info!("deleted {} records from tx_order: {}", deleted_count, from);
        Ok(())
    }

    pub fn get_stats(&self) -> anyhow::Result<IndexerStats> {
        let rtxn = self.db_env.read_txn()?;
        let db = self.db;
        let count = db.iter(&rtxn)?.count();
        rtxn.commit()?;
        Ok(IndexerStats {
            total_tx_count: count as u64,
            last_tx_order: self.last_tx_order,
            last_block_number: self.last_block_number,
        })
    }
}

impl IndexCommand {
    pub fn execute(self) -> anyhow::Result<()> {
        if self.index_file_path.is_some() {
            return Indexer::load_or_dump(
                self.index_path,
                self.index_file_path.unwrap(),
                self.dump,
            );
        }

        let db_path = self.index_path.clone();
        let reset_from = self.reset_from;
        let mut indexer = Indexer::new(db_path, reset_from)?;

        info!("indexer stats after reset: {:?}", indexer.get_stats()?);

        let segment_dir = self.segment_dir.ok_or(anyhow!("segment-dir is required"))?;

        let ledger_tx_loader = LedgerTxGetter::new(segment_dir)?;
        let mut block_number = indexer.last_block_number; // avoiding partial indexing
        let mut expected_tx_order = indexer.last_tx_order + 1;
        let stop_at = if let Some(max_block_number) = self.max_block_number {
            max(max_block_number, ledger_tx_loader.get_max_chunk_id())
        } else {
            ledger_tx_loader.get_max_chunk_id()
        };

        let db = indexer.db;
        let mut wtxn = indexer.db_env.write_txn()?;

        let mut done_block = 0;
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
                if tx_order == indexer.last_tx_order + 1 {
                    info!(
                        "begin to index block: {}, tx_order: {}",
                        block_number, tx_order
                    );
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
            done_block += 1;
            if done_block % 1000 == 0 {
                wtxn.commit()?;
                wtxn = indexer.db_env.write_txn()?;
                info!(
                    "done: block_cnt: {}; next_block_number: {}",
                    done_block, block_number
                );
            }
        }
        wtxn.commit()?;
        indexer.db_env.force_sync()?;

        indexer.init_cursor()?;

        info!("indexer stats after job: {:?}", indexer.get_stats()?);

        Ok(())
    }
}
