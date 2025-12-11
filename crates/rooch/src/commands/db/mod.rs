// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::commands::db::commands::best_rollback::BestRollbackCommand;
use crate::commands::db::commands::changeset::ChangesetCommand;
use crate::commands::db::commands::check_refcount::CheckRefcountCommand;
use crate::commands::db::commands::cp_cf::CpCfCommand;
use crate::commands::db::commands::delete_benchmark::DeleteBenchmarkCommand;
use crate::commands::db::commands::drop::DropCommand;
use crate::commands::db::commands::dump_state::DumpStateCommand;
use crate::commands::db::commands::gc::GCCommand;
use crate::commands::db::commands::generate_db_checkpoint::GenerateDBCheckPointCommand;
use crate::commands::db::commands::get_accumulator_leaf_by_index::GetAccumulatorLeafByIndexCommand;
use crate::commands::db::commands::get_changeset_by_order::GetChangesetByOrderCommand;
use crate::commands::db::commands::get_execution_info_by_hash::GetExecutionInfoByHashCommand;
use crate::commands::db::commands::get_sequencer_info::GetSequencerInfoCommand;
use crate::commands::db::commands::get_tx_by_order::GetTxByOrderCommand;
use crate::commands::db::commands::import_state::ImportStateCommand;
use crate::commands::db::commands::list_anomaly::ListAnomaly;
use crate::commands::db::commands::recycle::RecycleCommand;
use crate::commands::db::commands::repair::RepairCommand;
use crate::commands::db::commands::revert::RevertCommand;
use crate::commands::db::commands::rocksdb_gc::RocksDBGcCommand;
use crate::commands::db::commands::rocksdb_stats::RocksDBStatsCommand;
use crate::commands::db::commands::stat_changeset::StatChangesetCommand;
use crate::commands::db::commands::state_prune::StatePruneCommand;
use crate::commands::db::commands::verify_order::VerifyOrderCommand;
use async_trait::async_trait;
use clap::Parser;
use commands::rollback::RollbackCommand;
use rooch_types::error::RoochResult;

pub mod commands;

/// DB Commands
#[derive(Parser)]
pub struct DB {
    #[clap(subcommand)]
    cmd: DBCommand,
}

#[async_trait]
impl CommandAction<String> for DB {
    async fn execute(self) -> RoochResult<String> {
        match self.cmd {
            DBCommand::Revert(revert) => revert.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DBCommand::Rollback(rollback) => rollback.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DBCommand::Drop(drop) => drop.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DBCommand::Repair(repair) => repair.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DBCommand::GetTxByOrder(get_tx_by_order) => get_tx_by_order
                .execute()
                .map(|resp| serde_json::to_string(&resp).expect("Failed to serialize response")),
            DBCommand::GetChangesetByOrder(get_changeset_by_order) => get_changeset_by_order
                .execute()
                .await
                .map(|resp| serde_json::to_string(&resp).expect("Failed to serialize response")),
            DBCommand::GetExecutionInfoByHash(get_execution_info_by_hash) => {
                get_execution_info_by_hash
                    .execute()
                    .map(|resp| serde_json::to_string(&resp).expect("Failed to serialize response"))
            }
            DBCommand::BestRollback(best_rollback) => best_rollback.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DBCommand::ListAnomaly(list_anomaly) => list_anomaly.execute().map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DBCommand::CpCf(cp_cf) => cp_cf.execute().map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DBCommand::Changeset(changeset) => changeset.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DBCommand::StatChangeset(stat_changeset) => {
                stat_changeset.execute().await.map(|resp| {
                    serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
                })
            }
            DBCommand::VerifyOrder(verify_order) => verify_order.execute().map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DBCommand::GetSequencerInfo(get_sequencer_info) => {
                get_sequencer_info.execute().map(|resp| {
                    serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
                })
            }
            DBCommand::GetAccumulatorLeafByIndex(get_accumulator_leaf_by_index) => {
                get_accumulator_leaf_by_index.execute().map(|resp| {
                    serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
                })
            }
            DBCommand::DumpFromStateDB(dump_state_db) => {
                dump_state_db.execute().await.map(|resp| {
                    serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
                })
            }
            DBCommand::ImportToStateDB(import_state) => import_state.execute().await.map(|resp| {
                serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
            }),
            DBCommand::RocksdbStats(stats) => stats.execute().await,
            DBCommand::RocksdbGc(gc) => gc.execute().await,
            DBCommand::DeleteBenchmark(bench) => bench.execute().await,
            DBCommand::GenerateDBCheckPoint(generate_db_checkpoint) => {
                generate_db_checkpoint.execute().await.map(|resp| {
                    serde_json::to_string_pretty(&resp).expect("Failed to serialize response")
                })
            }
            DBCommand::CheckRefcount(check) => check.execute().await,
            DBCommand::GC(gc) => gc.execute().await,
            DBCommand::Recycle(recycle) => recycle.execute().await,
            DBCommand::StatePrune(state_prune) => state_prune.execute().await,
        }
    }
}

#[derive(clap::Subcommand)]
#[clap(name = "db")]
pub enum DBCommand {
    Revert(RevertCommand),
    Rollback(RollbackCommand),
    Drop(DropCommand),
    Repair(RepairCommand),
    GetTxByOrder(GetTxByOrderCommand),
    GetChangesetByOrder(GetChangesetByOrderCommand),
    GetExecutionInfoByHash(GetExecutionInfoByHashCommand),
    BestRollback(BestRollbackCommand),
    ListAnomaly(ListAnomaly),
    CpCf(CpCfCommand),
    Changeset(ChangesetCommand),
    StatChangeset(StatChangesetCommand),
    VerifyOrder(VerifyOrderCommand),
    GetSequencerInfo(GetSequencerInfoCommand),
    GetAccumulatorLeafByIndex(GetAccumulatorLeafByIndexCommand),
    DumpFromStateDB(DumpStateCommand),
    ImportToStateDB(ImportStateCommand),
    RocksdbStats(RocksDBStatsCommand),
    RocksdbGc(RocksDBGcCommand),
    DeleteBenchmark(DeleteBenchmarkCommand),
    GenerateDBCheckPoint(GenerateDBCheckPointCommand),
    CheckRefcount(CheckRefcountCommand),
    GC(GCCommand),
    Recycle(RecycleCommand),
    StatePrune(StatePruneCommand),
}
