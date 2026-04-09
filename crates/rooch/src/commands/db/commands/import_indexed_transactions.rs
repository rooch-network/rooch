// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::{open_rooch_db, open_rooch_db_readonly};
use anyhow::{Context, Result};
use clap::Parser;
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use moveos_common::utils::to_bytes;
use moveos_store::transaction_store::TransactionStore as TxExecutionInfoStore;
use moveos_types::h256::H256;
use moveos_types::transaction::TransactionExecutionInfo;
use rooch_config::{RoochOpt, R_OPT_NET_HELP};
use rooch_db::RoochDB;
use rooch_indexer::models::transactions::StoredTransaction;
use rooch_indexer::schema::transactions::dsl as tx_dsl;
use rooch_store::TRANSACTION_COLUMN_FAMILY_NAME;
use rooch_types::error::RoochResult;
use rooch_types::rooch_network::RoochChainID;
use rooch_types::transaction::LedgerTransaction;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::info;

const DEFAULT_BATCH_SIZE: usize = 1000;

/// Import missing transaction history referenced by the target indexer.
#[derive(Debug, Parser)]
pub struct ImportIndexedTransactionsCommand {
    #[clap(long = "source-data-dir")]
    pub source_data_dir: PathBuf,

    #[clap(long = "target-data-dir", short = 'd')]
    pub target_data_dir: Option<PathBuf>,

    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(long, default_value_t = DEFAULT_BATCH_SIZE)]
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct ImportIndexedTransactionsReport {
    pub batches: u64,
    pub indexed_rows_scanned: u64,
    pub missing_target_transactions: u64,
    pub missing_target_execution_infos: u64,
    pub skipped_non_l2_transactions: u64,
    pub source_transactions_missing: u64,
    pub source_execution_infos_missing: u64,
    pub imported_transactions: u64,
    pub imported_execution_infos: u64,
}

impl ImportIndexedTransactionsCommand {
    pub async fn execute(self) -> RoochResult<ImportIndexedTransactionsReport> {
        let source_dir = self.source_data_dir.clone();
        let target_dir = self.target_data_dir.clone();
        let chain_id = self.chain_id;
        let batch_size = self.batch_size.max(1);

        let (_source_root, source_db, _source_opened_at) =
            open_rooch_db_readonly(Some(source_dir), chain_id.clone());
        let (_target_root, target_db, _target_opened_at) =
            open_rooch_db(target_dir.clone(), chain_id.clone());

        let target_indexer_dir = derive_indexer_dir(target_dir, chain_id)?;

        Ok(run_import_indexed_transactions(
            &source_db,
            &target_db,
            &target_indexer_dir,
            batch_size,
        )?)
    }
}

fn derive_indexer_dir(
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
) -> Result<PathBuf> {
    let opt = RoochOpt::new_with_default(base_data_dir, chain_id, None)?;
    Ok(opt.store_config().get_indexer_dir())
}

fn load_indexed_transaction_batch(
    target_indexer_dir: &Path,
    after_tx_order: i64,
    batch_size: usize,
) -> Result<Vec<StoredTransaction>> {
    let tx_db_path = target_indexer_dir.join("transactions");
    let tx_db_url = tx_db_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid target transactions indexer path"))?;
    let mut conn = SqliteConnection::establish(tx_db_url)
        .with_context(|| format!("open target transactions sqlite {}", tx_db_path.display()))?;

    tx_dsl::transactions
        .select((
            tx_dsl::tx_hash,
            tx_dsl::tx_order,
            tx_dsl::sequence_number,
            tx_dsl::sender,
            tx_dsl::action_type,
            tx_dsl::auth_validator_id,
            tx_dsl::gas_used,
            tx_dsl::status,
            tx_dsl::created_at,
        ))
        .filter(tx_dsl::tx_order.gt(after_tx_order))
        .order_by(tx_dsl::tx_order.asc())
        .limit(batch_size as i64)
        .load::<StoredTransaction>(&mut conn)
        .context("load target indexed transaction batch")
}

fn run_import_indexed_transactions(
    source_db: &RoochDB,
    target_db: &RoochDB,
    target_indexer_dir: &Path,
    batch_size: usize,
) -> Result<ImportIndexedTransactionsReport> {
    let target_inner = target_db
        .rooch_store
        .store_instance
        .db()
        .ok_or_else(|| anyhow::anyhow!("failed to access target RocksDB instance"))?
        .inner();

    let tx_cf = target_inner
        .cf_handle(TRANSACTION_COLUMN_FAMILY_NAME)
        .ok_or_else(|| anyhow::anyhow!("Target CF not found: {}", TRANSACTION_COLUMN_FAMILY_NAME))?;
    let exec_cf = target_inner
        .cf_handle(moveos_store::TRANSACTION_EXECUTION_INFO_COLUMN_FAMILY_NAME)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Target CF not found: {}",
                moveos_store::TRANSACTION_EXECUTION_INFO_COLUMN_FAMILY_NAME
            )
        })?;

    let mut report = ImportIndexedTransactionsReport::default();
    let mut last_seen_order = -1_i64;

    loop {
        let batch = load_indexed_transaction_batch(target_indexer_dir, last_seen_order, batch_size)?;
        if batch.is_empty() {
            break;
        }
        report.batches += 1;
        report.indexed_rows_scanned += batch.len() as u64;
        last_seen_order = batch
            .last()
            .map(|tx| tx.tx_order)
            .ok_or_else(|| anyhow::anyhow!("empty batch has no last tx order"))?;

        let tx_hashes = batch
            .iter()
            .map(|row| H256::from_str(row.tx_hash.as_str()))
            .collect::<Result<Vec<_>, _>>()
            .context("decode target indexer tx hashes")?;

        let target_txs = target_db
            .rooch_store
            .transaction_store
            .get_transactions(tx_hashes.clone())
            .context("load target transactions by hash")?;
        let target_exec_infos = target_db
            .moveos_store
            .get_transaction_store()
            .multi_get_tx_execution_infos(tx_hashes.clone())
            .context("load target execution infos by hash")?;
        let source_txs = source_db
            .rooch_store
            .transaction_store
            .get_transactions(tx_hashes.clone())
            .context("load source transactions by hash")?;
        let source_exec_infos = source_db
            .moveos_store
            .get_transaction_store()
            .multi_get_tx_execution_infos(tx_hashes.clone())
            .context("load source execution infos by hash")?;

        let mut write_batch = rocksdb::WriteBatch::default();
        let mut writes_in_batch = 0u64;

        for (((tx_hash, target_tx), target_exec), (source_tx, source_exec)) in tx_hashes
            .into_iter()
            .zip(target_txs)
            .zip(target_exec_infos)
            .zip(source_txs.into_iter().zip(source_exec_infos))
        {
            let needs_tx = target_tx.is_none();
            let needs_exec = target_exec.is_none();

            if needs_tx {
                report.missing_target_transactions += 1;
            }
            if needs_exec {
                report.missing_target_execution_infos += 1;
            }
            if !needs_tx && !needs_exec {
                continue;
            }

            if needs_tx {
                match source_tx {
                    Some(tx) => {
                        if !tx.data.is_l2_tx() {
                            report.skipped_non_l2_transactions += 1;
                            continue;
                        }
                        write_transaction(&mut write_batch, &tx_cf, tx_hash, &tx)?;
                        report.imported_transactions += 1;
                        writes_in_batch += 1;
                    }
                    None => {
                        report.source_transactions_missing += 1;
                    }
                }
            }

            if needs_exec {
                match source_exec {
                    Some(execution_info) => {
                        write_execution_info(&mut write_batch, &exec_cf, tx_hash, &execution_info)?;
                        report.imported_execution_infos += 1;
                        writes_in_batch += 1;
                    }
                    None => {
                        report.source_execution_infos_missing += 1;
                    }
                }
            }
        }

        if writes_in_batch > 0 {
            target_inner
                .write(write_batch)
                .context("write imported transactions batch to target DB")?;
        }

        info!(
            "Imported indexed transactions batch {}: scanned={}, imported_txs={}, imported_exec_infos={}, skipped_non_l2={}, missing_target_txs={}, missing_target_exec_infos={}",
            report.batches,
            report.indexed_rows_scanned,
            report.imported_transactions,
            report.imported_execution_infos,
            report.skipped_non_l2_transactions,
            report.missing_target_transactions,
            report.missing_target_execution_infos,
        );
    }

    Ok(report)
}

fn write_transaction(
    batch: &mut rocksdb::WriteBatch,
    tx_cf: &impl rocksdb::AsColumnFamilyRef,
    tx_hash: H256,
    tx: &LedgerTransaction,
) -> Result<()> {
    batch.put_cf(tx_cf, to_bytes(&tx_hash)?, bcs::to_bytes(tx)?);
    Ok(())
}

fn write_execution_info(
    batch: &mut rocksdb::WriteBatch,
    exec_cf: &impl rocksdb::AsColumnFamilyRef,
    tx_hash: H256,
    execution_info: &TransactionExecutionInfo,
) -> Result<()> {
    batch.put_cf(exec_cf, to_bytes(&tx_hash)?, bcs::to_bytes(execution_info)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::insert_into;
    use metrics::RegistryService;
    use move_core_types::account_address::AccountAddress;
    use move_core_types::vm_status::KeptVMStatus;
    use moveos_types::moveos_std::object::ObjectMeta;
    use moveos_types::moveos_std::tx_context::TxContext;
    use moveos_types::transaction::{TransactionExecutionInfo, VerifiedMoveOSTransaction};
    use rooch_indexer::models::transactions::{escape_transaction, StoredTransaction};
    use rooch_indexer::schema::transactions;
    use rooch_indexer::IndexerStore;
    use rooch_types::indexer::transaction::IndexerTransaction;
    use rooch_types::rooch_network::BuiltinChainID;
    use rooch_types::test_utils::{random_ledger_transaction, random_verified_move_action};

    fn init_test_db(base_data_dir: PathBuf) -> RoochDB {
        let opt = RoochOpt::new_with_default(
            Some(base_data_dir),
            Some(BuiltinChainID::Local.into()),
            None,
        )
        .unwrap();
        let registry = RegistryService::default();
        RoochDB::init(opt.store_config(), &registry.default_registry()).unwrap()
    }

    fn seed_source_records(
        source_db: &RoochDB,
        ledger_tx: &LedgerTransaction,
        execution_info: &TransactionExecutionInfo,
    ) {
        let db = source_db
            .rooch_store
            .store_instance
            .db()
            .unwrap()
            .inner();
        let tx_cf = db.cf_handle(TRANSACTION_COLUMN_FAMILY_NAME).unwrap();
        let exec_cf = db
            .cf_handle(moveos_store::TRANSACTION_EXECUTION_INFO_COLUMN_FAMILY_NAME)
            .unwrap();
        let tx_hash = execution_info.tx_hash;
        let mut batch = rocksdb::WriteBatch::default();
        batch.put_cf(&tx_cf, to_bytes(&tx_hash).unwrap(), bcs::to_bytes(ledger_tx).unwrap());
        batch.put_cf(
            &exec_cf,
            to_bytes(&tx_hash).unwrap(),
            bcs::to_bytes(execution_info).unwrap(),
        );
        db.write(batch).unwrap();
    }

    #[test]
    fn import_indexed_transactions_fills_missing_transaction_and_execution_info() {
        let source_dir = moveos_config::temp_dir();
        let target_dir = moveos_config::temp_dir();
        let source_db = init_test_db(source_dir.path().to_path_buf());
        let target_db = init_test_db(target_dir.path().to_path_buf());

        let mut ledger_tx = random_ledger_transaction();
        let tx_hash = ledger_tx.tx_hash();
        let execution_info = TransactionExecutionInfo::new(
            tx_hash,
            H256::random(),
            rand::random(),
            H256::random(),
            rand::random(),
            KeptVMStatus::Executed,
        );
        seed_source_records(&source_db, &ledger_tx, &execution_info);

        let tx_context = TxContext::new_readonly_ctx(AccountAddress::random());
        let move_action = random_verified_move_action();
        let verified_move_tx = VerifiedMoveOSTransaction {
            root: ObjectMeta::genesis_root(),
            ctx: tx_context,
            action: move_action,
        };
        let indexer_tx = IndexerTransaction::new(
            ledger_tx.clone(),
            execution_info.clone(),
            verified_move_tx.action.into(),
            verified_move_tx.ctx.clone(),
        )
        .unwrap();

        let target_indexer_dir = derive_indexer_dir(
            Some(target_dir.path().to_path_buf()),
            Some(BuiltinChainID::Local.into()),
        )
        .unwrap();
        let registry = RegistryService::default();
        let _indexer_store =
            IndexerStore::new(target_indexer_dir.clone(), &registry.default_registry()).unwrap();
        let tx_db_path = target_indexer_dir.join("transactions");
        let tx_db_url = tx_db_path.to_str().unwrap();
        let mut conn = SqliteConnection::establish(tx_db_url).unwrap();
        let stored_tx = escape_transaction(StoredTransaction::from(indexer_tx));
        insert_into(transactions::table)
            .values(&stored_tx)
            .execute(&mut conn)
            .unwrap();

        assert!(
            target_db
                .rooch_store
                .transaction_store
                .get_transaction_by_hash(tx_hash)
                .unwrap()
                .is_none()
        );
        assert!(
            target_db
                .moveos_store
                .get_transaction_store()
                .get_tx_execution_info(tx_hash)
                .unwrap()
                .is_none()
        );

        let report =
            run_import_indexed_transactions(&source_db, &target_db, &target_indexer_dir, 32)
                .unwrap();

        assert_eq!(report.indexed_rows_scanned, 1);
        assert_eq!(report.imported_transactions, 1);
        assert_eq!(report.imported_execution_infos, 1);
        assert_eq!(report.source_transactions_missing, 0);
        assert_eq!(report.source_execution_infos_missing, 0);

        let source_tx = source_db
            .rooch_store
            .transaction_store
            .get_transaction_by_hash(tx_hash)
            .unwrap();
        let target_tx = target_db
            .rooch_store
            .transaction_store
            .get_transaction_by_hash(tx_hash)
            .unwrap();
        assert_eq!(target_tx, source_tx);

        let source_execution_info = source_db
            .moveos_store
            .get_transaction_store()
            .get_tx_execution_info(tx_hash)
            .unwrap();
        let target_execution_info = target_db
            .moveos_store
            .get_transaction_store()
            .get_tx_execution_info(tx_hash)
            .unwrap();
        assert_eq!(target_execution_info, source_execution_info);
    }
}
