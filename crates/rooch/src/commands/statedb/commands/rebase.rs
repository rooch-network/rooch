// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{anyhow, bail, ensure, Result};
use clap::Parser;
use metrics::RegistryService;
use moveos_store::{MoveOSStore, StoreMeta as MoveOSStoreMeta};
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::{ObjectID, ObjectMeta};
use moveos_types::startup_info::StartupInfo;
use moveos_types::state::{FieldKey, ObjectState};
use moveos_types::state_resolver::{RootObjectResolver, StateResolver};
use raw_store::metrics::DBMetrics;
use raw_store::rocks::RocksDB;
use raw_store::StoreInstance;
use rocksdb::checkpoint::Checkpoint;
use rooch_config::store_config::{
    DEFAULT_DB_DIR, DEFAULT_DB_INDEXER_SUBDIR, DEFAULT_DB_STORE_SUBDIR,
};
use rooch_config::R_OPT_NET_HELP;
use rooch_db::RoochDB;
use rooch_indexer::store::traits::IndexerStoreTrait;
use rooch_indexer::IndexerStore;
use rooch_store::{RoochStore, StoreMeta as RoochStoreMeta};
use rooch_types::error::RoochResult;
use rooch_types::indexer::state::{IndexerObjectState, IndexerObjectStatesIndexGenerator};
use rooch_types::rooch_network::{BuiltinChainID, RoochChainID};
use serde::{Deserialize, Serialize};
use smt::{UpdateSet, SPARSE_MERKLE_PLACEHOLDER_HASH};

use crate::commands::statedb::commands::{apply_fields, apply_nodes};
use crate::utils::open_rooch_db_readonly;

const REBASE_ARTIFACT_VERSION: u64 = 1;
const REBASE_OBJECTS_FILE: &str = "objects.jsonl";
const REBASE_MANIFEST_FILE: &str = "manifest.json";
const DEFAULT_EXPORT_PAGE_SIZE: usize = 1024;
const DEFAULT_INDEXER_BATCH_SIZE: usize = 4096;
const SLIM_PROFILE_NAME: &str = "slim-public-mainnet-v1";

#[derive(Debug, Serialize, Deserialize)]
struct RebaseManifest {
    artifact_version: u64,
    source_chain_id: String,
    cutover_state_root: String,
    cutover_tx_order: u64,
    filter_profile: String,
    dropped_domains: Vec<String>,
    object_records: u64,
    field_entries: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct RebaseObjectRecord {
    object_id: String,
    fields: Vec<RebaseFieldRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RebaseFieldRecord {
    field_key: String,
    object_state: String,
}

#[derive(Debug, Serialize)]
pub struct RebaseExportSummary {
    pub output_dir: PathBuf,
    pub object_records: u64,
    pub field_entries: u64,
    pub cutover_state_root: String,
    pub cutover_tx_order: u64,
    pub dropped_domains: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RebaseBuildSummary {
    pub output_data_dir: PathBuf,
    pub output_store_dir: PathBuf,
    pub rebuilt_state_root: String,
    pub rebuilt_global_size: u64,
    pub rebuilt_objects: u64,
    pub rebuilt_indexer_objects: u64,
}

#[derive(Debug, Default)]
struct ExportStats {
    object_records: u64,
    field_entries: u64,
}

#[derive(Debug, Clone, Parser)]
pub struct RebaseExportCommand {
    #[clap(
        long,
        help = "Input RocksDB store dir (preferred for checkpoint-based runs)"
    )]
    pub input_store_dir: Option<PathBuf>,

    #[clap(long = "data-dir", short = 'd')]
    pub base_data_dir: Option<PathBuf>,

    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: Option<RoochChainID>,

    #[clap(long, short = 'o')]
    pub output_dir: PathBuf,

    #[clap(long, default_value_t = DEFAULT_EXPORT_PAGE_SIZE)]
    pub page_size: usize,
}

#[derive(Debug, Clone, Parser)]
pub struct RebaseBuildCommand {
    #[clap(
        long,
        help = "Input RocksDB store dir (ideally a checkpoint store dir)"
    )]
    pub input_store_dir: PathBuf,

    #[clap(long, short = 'i')]
    pub artifact_dir: PathBuf,

    #[clap(long = "output-data-dir", short = 'o')]
    pub output_data_dir: PathBuf,

    #[clap(long, short = 'n', help = R_OPT_NET_HELP)]
    pub chain_id: RoochChainID,

    #[clap(long, default_value_t = DEFAULT_INDEXER_BATCH_SIZE)]
    pub indexer_batch_size: usize,
}

impl RebaseExportCommand {
    pub async fn execute(self) -> RoochResult<RebaseExportSummary> {
        self.execute_impl().map_err(Into::into)
    }

    fn execute_impl(self) -> Result<RebaseExportSummary> {
        if self.input_store_dir.is_some() {
            ensure!(
                self.chain_id.is_some(),
                "--chain-id is required when --input-store-dir is specified"
            );
        }

        let (root, moveos_store, rooch_store, source_chain_id) = open_input_source(
            self.input_store_dir.clone(),
            self.base_data_dir.clone(),
            self.chain_id.clone(),
            true,
        )?;

        ensure!(self.page_size > 0, "page_size must be greater than zero");

        if self.output_dir.exists() {
            bail!("output dir already exists: {:?}", self.output_dir);
        }
        fs::create_dir_all(&self.output_dir)?;

        let objects_path = self.output_dir.join(REBASE_OBJECTS_FILE);
        let objects_file = File::create(&objects_path)?;
        let mut writer = BufWriter::new(objects_file);

        let resolver = RootObjectResolver::new(root.clone(), &moveos_store);
        let root_state = ObjectState::new_root(root.clone());
        let mut stats = ExportStats::default();

        let retained = export_object_record_recursive(
            &resolver,
            &root_state,
            self.page_size,
            &mut writer,
            &mut stats,
        )?;
        ensure!(retained, "root object produced no retained fields");
        writer.flush()?;

        let sequencer_info = rooch_store
            .get_meta_store()
            .get_sequencer_info()?
            .ok_or_else(|| anyhow!("sequencer info not found in input source"))?;

        let manifest = RebaseManifest {
            artifact_version: REBASE_ARTIFACT_VERSION,
            source_chain_id: source_chain_id.to_string(),
            cutover_state_root: format!("{:#x}", root.state_root()),
            cutover_tx_order: sequencer_info.last_order,
            filter_profile: SLIM_PROFILE_NAME.to_string(),
            dropped_domains: Vec::new(),
            object_records: stats.object_records,
            field_entries: stats.field_entries,
        };
        serde_json::to_writer_pretty(
            File::create(self.output_dir.join(REBASE_MANIFEST_FILE))?,
            &manifest,
        )?;

        Ok(RebaseExportSummary {
            output_dir: self.output_dir,
            object_records: stats.object_records,
            field_entries: stats.field_entries,
            cutover_state_root: manifest.cutover_state_root,
            cutover_tx_order: manifest.cutover_tx_order,
            dropped_domains: manifest.dropped_domains,
        })
    }
}

impl RebaseBuildCommand {
    pub async fn execute(self) -> RoochResult<RebaseBuildSummary> {
        self.execute_impl().map_err(Into::into)
    }

    fn execute_impl(self) -> Result<RebaseBuildSummary> {
        ensure!(
            self.indexer_batch_size > 0,
            "indexer_batch_size must be greater than zero"
        );

        let manifest_path = self.artifact_dir.join(REBASE_MANIFEST_FILE);
        let objects_path = self.artifact_dir.join(REBASE_OBJECTS_FILE);
        ensure!(
            manifest_path.is_file(),
            "missing manifest: {:?}",
            manifest_path
        );
        ensure!(
            objects_path.is_file(),
            "missing object records: {:?}",
            objects_path
        );

        let manifest: RebaseManifest = serde_json::from_reader(File::open(&manifest_path)?)?;
        ensure!(
            manifest.artifact_version == REBASE_ARTIFACT_VERSION,
            "unsupported artifact version: {}",
            manifest.artifact_version
        );
        let artifact_chain_id = RoochChainID::from_str(&manifest.source_chain_id)?;
        ensure!(
            artifact_chain_id == self.chain_id,
            "artifact chain id {} does not match build target chain id {}",
            artifact_chain_id,
            self.chain_id
        );

        let (output_store_dir, output_indexer_dir) =
            derive_output_paths(&self.output_data_dir, &self.chain_id);
        ensure!(
            !output_store_dir.exists(),
            "output store dir already exists: {:?}",
            output_store_dir
        );
        ensure!(
            !output_indexer_dir.exists(),
            "output indexer dir already exists: {:?}",
            output_indexer_dir
        );
        fs::create_dir_all(
            output_store_dir
                .parent()
                .ok_or_else(|| anyhow!("invalid output store dir"))?,
        )?;

        create_store_checkpoint(&self.input_store_dir, &output_store_dir)?;

        let registry_service = RegistryService::default();
        let output_opt = rooch_config::RoochOpt::new_with_default(
            Some(self.output_data_dir.clone()),
            Some(self.chain_id.clone()),
            None,
        )
        .map_err(|e| anyhow!("failed to initialize output config: {}", e))?;
        let rooch_db = RoochDB::init(
            output_opt.store_config(),
            &registry_service.default_registry(),
        )?;

        clear_state_nodes(&rooch_db.moveos_store)?;

        let reader = BufReader::new(File::open(&objects_path)?);
        let mut rebuilt_roots: BTreeMap<ObjectID, H256> = BTreeMap::new();
        let mut rebuilt_objects = 0u64;
        let mut rebuilt_root = None;
        let mut rebuilt_global_size = None;
        let mut current_object_id: Option<ObjectID> = None;
        let mut current_fields = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let record: RebaseObjectRecord = serde_json::from_str(&line)?;
            let object_id = ObjectID::from_str(&record.object_id)?;
            if current_object_id
                .as_ref()
                .is_some_and(|current| current != &object_id)
            {
                let current_id = current_object_id
                    .take()
                    .ok_or_else(|| anyhow!("missing current object while rebuilding"))?;
                let field_count = current_fields.len() as u64;
                let new_state_root = rebuild_object_fields(
                    &rooch_db.moveos_store,
                    &mut rebuilt_roots,
                    &current_id,
                    std::mem::take(&mut current_fields),
                )?;
                rebuilt_roots.insert(current_id.clone(), new_state_root);
                rebuilt_objects += 1;

                if current_id.is_root() {
                    rebuilt_root = Some(new_state_root);
                    rebuilt_global_size = Some(field_count);
                }
            }

            if current_object_id.is_none() {
                current_object_id = Some(object_id);
            }
            current_fields.extend(record.fields);
        }

        if let Some(object_id) = current_object_id.take() {
            let field_count = current_fields.len() as u64;
            let new_state_root = rebuild_object_fields(
                &rooch_db.moveos_store,
                &mut rebuilt_roots,
                &object_id,
                std::mem::take(&mut current_fields),
            )?;
            rebuilt_roots.insert(object_id.clone(), new_state_root);
            rebuilt_objects += 1;

            if object_id.is_root() {
                rebuilt_root = Some(new_state_root);
                rebuilt_global_size = Some(field_count);
            }
        }

        let rebuilt_root =
            rebuilt_root.ok_or_else(|| anyhow!("root object record not found in artifact"))?;
        let rebuilt_global_size =
            rebuilt_global_size.ok_or_else(|| anyhow!("root object size not found in artifact"))?;

        rooch_db
            .moveos_store
            .config_store
            .save_startup_info(StartupInfo::new(rebuilt_root, rebuilt_global_size))?;

        let rebuilt_indexer_objects = rebuild_indexer_from_state(
            &rooch_db.moveos_store,
            &rooch_db.indexer_store,
            ObjectMeta::root_metadata(rebuilt_root, rebuilt_global_size),
            manifest.cutover_tx_order,
            self.indexer_batch_size,
        )?;

        let summary = RebaseBuildSummary {
            output_data_dir: self.output_data_dir,
            output_store_dir,
            rebuilt_state_root: format!("{:#x}", rebuilt_root),
            rebuilt_global_size,
            rebuilt_objects,
            rebuilt_indexer_objects,
        };

        Ok(summary)
    }
}

fn open_input_source(
    input_store_dir: Option<PathBuf>,
    base_data_dir: Option<PathBuf>,
    chain_id: Option<RoochChainID>,
    readonly: bool,
) -> Result<(ObjectMeta, MoveOSStore, RoochStore, RoochChainID)> {
    match (input_store_dir, base_data_dir, chain_id) {
        (Some(store_dir), None, Some(chain_id)) => {
            let (root, moveos_store, rooch_store) =
                open_stores_from_store_dir(&store_dir, readonly)?;
            Ok((root, moveos_store, rooch_store, chain_id))
        }
        (Some(_), None, None) => {
            bail!("--chain-id is required when --input-store-dir is specified")
        }
        (None, None, _) => bail!("Either --input-store-dir or --data-dir must be specified"),
        (None, base_data_dir, chain_id) => {
            let chain_id = chain_id.unwrap_or_else(|| BuiltinChainID::Local.into());
            let (_opened_root, rooch_db, _start_time) = if readonly {
                open_rooch_db_readonly(base_data_dir, Some(chain_id.clone()))
            } else {
                crate::utils::open_rooch_db(base_data_dir, Some(chain_id.clone()))
            };
            let root = rooch_db
                .latest_root()?
                .ok_or_else(|| anyhow!("startup_info not found in input source"))?;
            Ok((root, rooch_db.moveos_store, rooch_db.rooch_store, chain_id))
        }
        (Some(_), Some(_), _) => bail!("Specify either --input-store-dir or --data-dir, not both"),
    }
}

fn open_stores_from_store_dir(
    store_dir: &Path,
    readonly: bool,
) -> Result<(ObjectMeta, MoveOSStore, RoochStore)> {
    let registry_service = RegistryService::default();
    let registry = registry_service.default_registry();
    let cfs = all_column_families();
    let db_metrics = DBMetrics::get_or_init(&registry).clone();
    let rocksdb = if readonly {
        RocksDB::new_readonly(
            store_dir,
            cfs,
            moveos_config::store_config::RocksdbConfig::default(),
        )?
    } else {
        RocksDB::new(
            store_dir,
            cfs,
            moveos_config::store_config::RocksdbConfig::default(),
        )?
    };
    let instance = StoreInstance::new_db_instance(rocksdb, db_metrics);
    let moveos_store = MoveOSStore::new_with_instance(instance.clone(), &registry)?;
    let rooch_store = RoochStore::new_with_instance(instance, &registry)?;
    let root = moveos_store
        .config_store
        .get_startup_info()?
        .ok_or_else(|| anyhow!("startup_info not found in {:?}", store_dir))?
        .into_root_metadata();
    Ok((root, moveos_store, rooch_store))
}

fn all_column_families() -> Vec<&'static str> {
    let mut column_families = MoveOSStoreMeta::get_column_family_names().to_vec();
    column_families.extend_from_slice(RoochStoreMeta::get_column_family_names());

    let mut seen = HashSet::new();
    column_families
        .into_iter()
        .filter(|cf| seen.insert(*cf))
        .collect()
}

fn export_object_record_recursive<R: StateResolver>(
    resolver: &R,
    object_state: &ObjectState,
    page_size: usize,
    writer: &mut BufWriter<File>,
    stats: &mut ExportStats,
) -> Result<bool> {
    if !object_state.metadata.has_fields() {
        return Ok(false);
    }

    let object_id = object_state.metadata.id.clone();
    let mut cursor = None;
    let mut object_field_entries = 0u64;
    let mut wrote_record = false;

    loop {
        let page = resolver.list_fields(&object_id, cursor, page_size)?;
        if page.is_empty() {
            break;
        }

        let mut fields = Vec::with_capacity(page.len());
        for (field_key, mut child_state) in page.iter().cloned() {
            if child_state.metadata.has_fields() {
                let child_has_fields = export_object_record_recursive(
                    resolver,
                    &child_state,
                    page_size,
                    writer,
                    stats,
                )?;
                if !child_has_fields {
                    child_state.metadata.size = 0;
                    child_state.metadata.state_root = None;
                }
            }

            fields.push(RebaseFieldRecord {
                field_key: field_key.to_string(),
                object_state: child_state.to_string(),
            });
        }

        if !fields.is_empty() {
            let field_count = fields.len() as u64;
            let record = RebaseObjectRecord {
                object_id: object_id.to_string(),
                fields,
            };
            serde_json::to_writer(&mut *writer, &record)?;
            writer.write_all(b"\n")?;
            object_field_entries += field_count;
            wrote_record = true;
        }

        cursor = page.last().map(|(field_key, _)| *field_key);
        if page.len() < page_size {
            break;
        }
    }

    if !wrote_record {
        return Ok(false);
    }

    stats.object_records += 1;
    stats.field_entries += object_field_entries;
    Ok(true)
}

fn derive_output_paths(base_data_dir: &Path, chain_id: &RoochChainID) -> (PathBuf, PathBuf) {
    let chain_dir = base_data_dir.join(chain_id.chain_name());
    let rooch_db_dir = chain_dir.join(DEFAULT_DB_DIR);
    (
        rooch_db_dir.join(DEFAULT_DB_STORE_SUBDIR),
        rooch_db_dir.join(DEFAULT_DB_INDEXER_SUBDIR),
    )
}

fn create_store_checkpoint(input_store_dir: &Path, output_store_dir: &Path) -> Result<()> {
    let rocksdb = RocksDB::new(
        input_store_dir,
        all_column_families(),
        moveos_config::store_config::RocksdbConfig::default(),
    )?;
    rocksdb.flush_all()?;
    rocksdb.flush_wal(true)?;
    let checkpoint = Checkpoint::new(rocksdb.inner())?;
    checkpoint.create_checkpoint(output_store_dir)?;
    Ok(())
}

fn clear_state_nodes(output_store: &MoveOSStore) -> Result<()> {
    let node_store = output_store.get_state_node_store();
    let start = H256::zero();
    let end = H256::from_slice(&[0xFFu8; 32]);

    node_store.delete_range_nodes(start, end, true)?;
    node_store.delete_nodes_with_flush(vec![end], true)?;
    node_store.flush_and_compact()?;
    Ok(())
}

fn rebuild_object_fields(
    moveos_store: &MoveOSStore,
    rebuilt_roots: &mut BTreeMap<ObjectID, H256>,
    object_id: &ObjectID,
    fields: Vec<RebaseFieldRecord>,
) -> Result<H256> {
    let mut update_set = UpdateSet::new();

    for field in fields {
        let field_key = FieldKey::from_str(&field.field_key)?;
        let mut object_state = ObjectState::from_str(&field.object_state)?;
        if object_state.metadata.has_fields() {
            let child_id = object_state.metadata.id.clone();
            let rebuilt_child_root = rebuilt_roots.remove(&child_id).ok_or_else(|| {
                anyhow!(
                    "missing rebuilt child root for {} while building {}",
                    child_id,
                    object_id
                )
            })?;
            object_state.update_state_root(rebuilt_child_root);
        }
        update_set.put(field_key, object_state);
    }

    let mut tree_change_set =
        apply_fields(moveos_store, *SPARSE_MERKLE_PLACEHOLDER_HASH, update_set)?;
    let new_state_root = tree_change_set.state_root;
    apply_nodes(moveos_store, std::mem::take(&mut tree_change_set.nodes))?;
    Ok(new_state_root)
}

fn rebuild_indexer_from_state(
    moveos_store: &MoveOSStore,
    indexer_store: &IndexerStore,
    root: ObjectMeta,
    tx_order: u64,
    batch_size: usize,
) -> Result<u64> {
    let resolver = RootObjectResolver::new(root, moveos_store);
    let mut generator = IndexerObjectStatesIndexGenerator::default();
    let mut batch = Vec::with_capacity(batch_size);
    let mut total = 0u64;

    walk_object_tree(
        &resolver,
        &ObjectID::root(),
        DEFAULT_EXPORT_PAGE_SIZE,
        &mut |state| {
            if state.metadata.is_dynamic_field() {
                return Ok(());
            }
            let state_index = generator.get(&state.metadata.object_type);
            generator.incr(&state.metadata.object_type);
            batch.push(IndexerObjectState::new(
                state.metadata.clone(),
                tx_order,
                state_index,
            ));
            if batch.len() >= batch_size {
                indexer_store.persist_or_update_object_states(std::mem::take(&mut batch))?;
            }
            total += 1;
            Ok(())
        },
    )?;

    if !batch.is_empty() {
        indexer_store.persist_or_update_object_states(batch)?;
    }

    Ok(total)
}

fn walk_object_tree<R, F>(
    resolver: &R,
    object_id: &ObjectID,
    page_size: usize,
    visitor: &mut F,
) -> Result<()>
where
    R: StateResolver,
    F: FnMut(&ObjectState) -> Result<()>,
{
    let mut cursor = None;
    loop {
        let page = resolver.list_fields(object_id, cursor, page_size)?;
        if page.is_empty() {
            break;
        }

        for (_field_key, child_state) in page.iter() {
            visitor(child_state)?;
            if child_state.metadata.has_fields() {
                walk_object_tree(resolver, &child_state.metadata.id, page_size, visitor)?;
            }
        }

        cursor = page.last().map(|(field_key, _)| *field_key);
        if page.len() < page_size {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use accumulator::accumulator_info::AccumulatorInfo;
    use move_core_types::account_address::AccountAddress;
    use move_core_types::ident_str;
    use move_core_types::identifier::IdentStr;
    use moveos_types::genesis_info::GenesisInfo;
    use moveos_types::moveos_std::object;
    use moveos_types::moveos_std::object::GENESIS_STATE_ROOT;
    use moveos_types::state::{MoveState, MoveStructState, MoveStructType, MoveType};
    use rooch_types::sequencer::SequencerInfo;
    use tokio::runtime::Runtime;

    use crate::commands::statedb::commands::init_rooch_db;

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    struct TestContainer {
        value: u64,
    }

    impl MoveStructType for TestContainer {
        const ADDRESS: AccountAddress = AccountAddress::TWO;
        const MODULE_NAME: &'static IdentStr = ident_str!("rebase_test");
        const STRUCT_NAME: &'static IdentStr = ident_str!("TestContainer");
    }

    impl MoveStructState for TestContainer {
        fn struct_layout() -> move_core_types::value::MoveStructLayout {
            move_core_types::value::MoveStructLayout::new(vec![u64::type_layout()])
        }
    }

    fn test_object_state(object_id: ObjectID, value: u64) -> ObjectState {
        let metadata = ObjectMeta::genesis_meta(object_id, TestContainer::type_tag());
        ObjectState::new_with_struct(metadata, TestContainer { value }).unwrap()
    }

    fn test_shared_container_with_root(
        object_id: ObjectID,
        state_root: H256,
        size: u64,
    ) -> ObjectState {
        let mut metadata = ObjectMeta::genesis_meta(object_id, TestContainer::type_tag());
        metadata.to_shared();
        metadata.state_root = Some(state_root);
        metadata.size = size;
        ObjectState::new_with_struct(metadata, TestContainer { value: size }).unwrap()
    }

    fn test_utxo_store_id() -> Result<ObjectID> {
        ObjectID::from_str("0xf74d177bfec2d8de0c4893f6502d3e5b55f12f75e158d53b035dcbe33782ef16")
            .map_err(Into::into)
    }

    fn prepare_source_db(base_data_dir: &Path) -> Result<PathBuf> {
        let chain_id: RoochChainID = BuiltinChainID::Local.into();
        let rooch_db = init_rooch_db(Some(base_data_dir.to_path_buf()), Some(chain_id.clone()));

        let mut parent = test_shared_container_with_root(
            object::named_object_id(&TestContainer::struct_tag()),
            *GENESIS_STATE_ROOT,
            0,
        );
        let child_id = parent
            .metadata
            .id
            .child_id(FieldKey::derive_from_string("child"));
        let child_leaf_id = child_id.child_id(FieldKey::derive_from_string("leaf"));
        let child_leaf = test_object_state(child_leaf_id.clone(), 99);
        let mut child_updates = UpdateSet::new();
        child_updates.put(child_leaf_id.field_key(), child_leaf.clone());
        let mut child_tree = apply_fields(
            &rooch_db.moveos_store,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            child_updates,
        )?;
        apply_nodes(
            &rooch_db.moveos_store,
            std::mem::take(&mut child_tree.nodes),
        )?;
        let child_state =
            test_shared_container_with_root(child_id.clone(), child_tree.state_root, 1);

        let kept_leaf_id = parent
            .metadata
            .id
            .child_id(FieldKey::derive_from_string("kept"));
        let kept_leaf = test_object_state(kept_leaf_id.clone(), 7);
        let mut parent_updates = UpdateSet::new();
        parent_updates.put(child_id.field_key(), child_state.clone());
        parent_updates.put(kept_leaf_id.field_key(), kept_leaf.clone());
        let mut parent_tree = apply_fields(
            &rooch_db.moveos_store,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            parent_updates,
        )?;
        apply_nodes(
            &rooch_db.moveos_store,
            std::mem::take(&mut parent_tree.nodes),
        )?;
        parent.update_state_root(parent_tree.state_root);
        parent.metadata.size = 2;

        let utxo_store_id = test_utxo_store_id()?;
        let mut utxo_store = test_shared_container_with_root(utxo_store_id, *GENESIS_STATE_ROOT, 0);
        let dropped_leaf_id = utxo_store
            .metadata
            .id
            .child_id(FieldKey::derive_from_string("drop"));
        let dropped_leaf = test_object_state(dropped_leaf_id.clone(), 1);
        let mut utxo_updates = UpdateSet::new();
        utxo_updates.put(dropped_leaf_id.field_key(), dropped_leaf);
        let mut utxo_tree = apply_fields(
            &rooch_db.moveos_store,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            utxo_updates,
        )?;
        apply_nodes(&rooch_db.moveos_store, std::mem::take(&mut utxo_tree.nodes))?;
        utxo_store.update_state_root(utxo_tree.state_root);
        utxo_store.metadata.size = 1;

        let root_leaf_id = ObjectID::root().child_id(FieldKey::derive_from_string("root-leaf"));
        let root_leaf = test_object_state(root_leaf_id.clone(), 1234);

        let mut root_updates = UpdateSet::new();
        root_updates.put(parent.metadata.id.field_key(), parent.clone());
        root_updates.put(utxo_store.metadata.id.field_key(), utxo_store.clone());
        root_updates.put(root_leaf_id.field_key(), root_leaf.clone());
        let mut root_tree = apply_fields(
            &rooch_db.moveos_store,
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            root_updates,
        )?;
        apply_nodes(&rooch_db.moveos_store, std::mem::take(&mut root_tree.nodes))?;

        rooch_db
            .moveos_store
            .config_store
            .save_genesis(GenesisInfo::new(H256::zero(), vec![]))?;
        rooch_db
            .moveos_store
            .config_store
            .save_startup_info(StartupInfo::new(root_tree.state_root, 3))?;
        rooch_db
            .rooch_store
            .get_meta_store()
            .save_sequencer_info(SequencerInfo::new(0, AccumulatorInfo::default()))?;

        std::mem::drop(rooch_db);
        let (input_store_dir, _input_indexer_dir) = derive_output_paths(base_data_dir, &chain_id);
        Ok(input_store_dir)
    }

    #[test]
    fn test_rebase_export_and_build_roundtrip() -> Result<()> {
        let source_dir = tempfile::tempdir()?;
        let checkpoint_dir = prepare_source_db(source_dir.path())?;

        let artifact_dir = source_dir.path().join("artifact");
        let runtime = Runtime::new()?;
        let export_summary = runtime.block_on(
            RebaseExportCommand {
                input_store_dir: Some(checkpoint_dir.clone()),
                base_data_dir: None,
                chain_id: Some(BuiltinChainID::Local.into()),
                output_dir: artifact_dir.clone(),
                page_size: 16,
            }
            .execute(),
        )?;
        assert!(export_summary.object_records >= 2);

        let output_data_dir = source_dir.path().join("output");
        let build_summary = runtime.block_on(
            RebaseBuildCommand {
                input_store_dir: checkpoint_dir,
                artifact_dir,
                output_data_dir: output_data_dir.clone(),
                chain_id: BuiltinChainID::Local.into(),
                indexer_batch_size: 16,
            }
            .execute(),
        )?;

        let (_root, rebuilt_db, _start_time) =
            crate::utils::open_rooch_db(Some(output_data_dir), Some(BuiltinChainID::Local.into()));
        let resolver = RootObjectResolver::new(
            rebuilt_db
                .latest_root()?
                .ok_or_else(|| anyhow!("missing rebuilt root"))?,
            &rebuilt_db.moveos_store,
        );

        let rebuilt_utxo = resolver
            .get_object(&test_utxo_store_id()?)?
            .ok_or_else(|| anyhow!("utxo object missing"))?;
        assert_eq!(rebuilt_utxo.metadata.size, 1);

        let retained_parent_id = object::named_object_id(&TestContainer::struct_tag());
        let rebuilt_parent = resolver
            .get_object(&retained_parent_id)?
            .ok_or_else(|| anyhow!("retained parent object missing"))?;
        assert_eq!(rebuilt_parent.metadata.size, 2);
        assert_ne!(
            build_summary.rebuilt_state_root,
            format!("{:#x}", *SPARSE_MERKLE_PLACEHOLDER_HASH)
        );

        Ok(())
    }
}
