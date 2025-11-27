// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::{open_rooch_db, open_rooch_db_readonly};
use clap::Parser;
use raw_store::CodecKVStore;
use rooch_pruner::recycle_bin::RecycleBinStore;
use rooch_types::error::RoochResult;
use serde_json;
use std::path::PathBuf;
use tracing::info;
use smt::jellyfish_merkle::node_type::Node;
use moveos_types::state::{FieldKey, ObjectState};
use moveos_types::h256::H256;

#[derive(Debug, serde::Serialize)]
struct DecodedNodeSummary {
    node_kind: String,
    field_key: Option<String>,
    object_id: Option<String>,
    object_type: Option<String>,
    state_root: Option<String>,
    value_len: Option<usize>,
}

/// Dump recycle bin record for a specific node hash
#[derive(Debug, Parser)]
pub struct RecycleDumpCommand {
    /// Base data dir, e.g. ~/.rooch
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    /// Chain ID
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,
    /// Node hash to dump
    #[clap(long)]
    pub hash: String,
    /// Output file
    #[clap(long, short = 'o')]
    pub output: Option<PathBuf>,
    /// Decode node bytes if possible (Node<FieldKey, ObjectState>); best-effort
    #[clap(long)]
    pub decode: bool,
}

impl RecycleDumpCommand {
    pub async fn execute(self) -> RoochResult<String> {
        let (_root_meta, rooch_db, _start) = open_rooch_db_readonly(
            self.base_data_dir.clone(),
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );

        let recycle_store = RecycleBinStore::new(
            rooch_db.moveos_store.get_node_recycle_store().clone(),
            10000,       // max_entries
            100_000_000, // max_bytes 100MB
        )?;

        // Parse hex string to bytes
        let hash_str = self.hash.strip_prefix("0x").unwrap_or(&self.hash);
        let hash_bytes = hex::decode(hash_str).map_err(|_| {
            rooch_types::error::RoochError::CommandArgumentError("Invalid hex format".to_string())
        })?;
        if hash_bytes.len() != 32 {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                "Hash must be 32 bytes".to_string(),
            ));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hash_bytes);
        let node_hash = moveos_types::h256::H256(arr);

        if let Some(record) = recycle_store.get_record(&node_hash)? {
            #[derive(Debug, serde::Serialize)]
            struct DumpOut {
                record: rooch_pruner::recycle_bin::RecycleRecord,
                #[serde(skip_serializing_if = "Option::is_none")]
                decoded: Option<DecodedNodeSummary>,
            }
            let mut decoded = None;
            if self.decode {
                decoded = decode_node(&record.bytes);
            }
            let out = DumpOut { record, decoded };
            let output = serde_json::to_string_pretty(&out)?;

            match self.output {
                Some(output_path) => {
                    std::fs::write(&output_path, output).map_err(|e| {
                        rooch_types::error::RoochError::UnexpectedError(e.to_string())
                    })?;
                    info!("Recycle bin record dumped to {:?}", output_path);
                    Ok("Record dumped successfully".to_string())
                }
                None => {
                    println!("{}", output);
                    Ok("Record dumped successfully".to_string())
                }
            }
        } else {
            eprintln!("No recycle bin record found for hash: {}", self.hash);
            Ok("No record found".to_string())
        }
    }
}

fn decode_node(bytes: &[u8]) -> Option<DecodedNodeSummary> {
    if let Ok(node) = Node::<FieldKey, ObjectState>::decode(bytes) {
        match node {
            Node::Internal(_internal) => Some(DecodedNodeSummary {
                node_kind: "Internal".to_string(),
                field_key: None,
                object_id: None,
                object_type: None,
                state_root: None,
                value_len: None,
            }),
            Node::Leaf(leaf) => {
                let fk = leaf.key();
                let val = leaf.value().origin.clone();
                let obj_id = val.metadata.id.to_string();
                let obj_type = val.metadata.object_type.to_canonical_string();
                let sr = val.metadata.state_root().to_string();
                Some(DecodedNodeSummary {
                    node_kind: "Leaf".to_string(),
                    field_key: Some(fk.to_string()),
                    object_id: Some(obj_id),
                    object_type: Some(obj_type),
                    state_root: Some(sr),
                    value_len: Some(val.value.len()),
                })
            }
            Node::Null => Some(DecodedNodeSummary {
                node_kind: "Null".to_string(),
                field_key: None,
                object_id: None,
                object_type: None,
                state_root: None,
                value_len: None,
            }),
        }
    } else {
        None
    }
}

/// Restore a node from recycle bin back to state_node
#[derive(Debug, Parser)]
pub struct RecycleRestoreCommand {
    /// Base data dir, e.g. ~/.rooch
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    /// Chain ID
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,
    /// Node hash to restore
    #[clap(long)]
    pub hash: String,
    /// Force restore (required)
    #[clap(long, short = 'f')]
    pub force: bool,
}

impl RecycleRestoreCommand {
    pub async fn execute(self) -> RoochResult<String> {
        if !self.force {
            eprintln!("Error: --force is required for restore operation");
            eprintln!("This operation will overwrite existing state_node data.");
            return Ok("Operation cancelled".to_string());
        }

        let (_root_meta, rooch_db, _start) = open_rooch_db(
            self.base_data_dir.clone(),
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );

        let recycle_store = RecycleBinStore::new(
            rooch_db.moveos_store.get_node_recycle_store().clone(),
            10000,
            100_000_000,
        )?;

        // Parse hex string to bytes
        let hash_str = self.hash.strip_prefix("0x").unwrap_or(&self.hash);
        let hash_bytes = hex::decode(hash_str).map_err(|_| {
            rooch_types::error::RoochError::CommandArgumentError("Invalid hex format".to_string())
        })?;
        if hash_bytes.len() != 32 {
            return Err(rooch_types::error::RoochError::CommandArgumentError(
                "Hash must be 32 bytes".to_string(),
            ));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hash_bytes);
        let node_hash = moveos_types::h256::H256(arr);

        if let Some(record) = recycle_store.get_record(&node_hash)? {
            // Restore node back to state_node store (not recycle store)
            rooch_db
                .moveos_store
                .get_state_node_store()
                .kv_put(node_hash, record.bytes)
                .map_err(|e| rooch_types::error::RoochError::UnexpectedError(e.to_string()))?;

            info!("Node {} restored to state_node store", self.hash);
            info!("Phase: {:?}", record.phase);
            info!("Deleted at: {}", record.deleted_at);

            // Optionally remove from recycle bin after successful restore
            // recycle_store.delete_record(&node_hash)?;

            Ok("Node restored successfully".to_string())
        } else {
            eprintln!("No recycle bin record found for hash: {}", self.hash);
            Ok("No record found".to_string())
        }
    }
}

/// Show recycle bin statistics
#[derive(Debug, Parser)]
pub struct RecycleStatCommand {
    /// Base data dir, e.g. ~/.rooch
    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,
    /// Chain ID
    #[clap(long, short = 'n')]
    pub chain_id: rooch_types::rooch_network::BuiltinChainID,
    /// Show detailed listing
    #[clap(long)]
    pub detailed: bool,
}

impl RecycleStatCommand {
    pub async fn execute(self) -> RoochResult<String> {
        let (_root_meta, rooch_db, _start) = open_rooch_db_readonly(
            self.base_data_dir.clone(),
            Some(rooch_types::rooch_network::RoochChainID::Builtin(
                self.chain_id,
            )),
        );

        let recycle_store = RecycleBinStore::new(
            rooch_db.moveos_store.get_node_recycle_store().clone(),
            10000,
            100_000_000,
        )?;

        let stats = recycle_store.get_stats();
        println!("{}", stats);

        if self.detailed {
            // Note: This would require implementing list_entries method
            println!("\nDetailed listing not yet implemented");
        }

        Ok("Statistics displayed".to_string())
    }
}
