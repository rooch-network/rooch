use anyhow::Result;
use moveos_types::state::{FieldKey, ObjectState};
use moveos_types::h256::H256;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{info, warn};
use moveos_store::MoveOSStore;
use moveos_types::moveos_std::object::ObjectID;

const PRUNING_BATCH_SIZE: usize = 1000;
const PRUNING_INTERVAL: Duration = Duration::from_secs(3600); // 1 hour
const PRUNING_TIME_THRESHOLD: u64 = 30 * 24 * 3600;

#[derive(Clone)]
pub struct StateRootInfo {
    pub state_root: H256,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct ObjectStatePruneInfo {
    pub prune_hash: H256,
    pub timestamp: u64, // object state update timestamp
}

pub struct StatePruner {
    // node_store: Arc<dyn crate::state_store::NodeDBStore>,
    // state_store: Arc<StateDBStore>,
    moveos_store: Arc<MoveOSStore>,
    latest_state_root: H256,
    // latest_state_map: Arc<RwLock<HashMap<FieldKey, ObjectStatePruneInfo>>>, // field_key -> ObjectStatePruneInfo hash
    latest_state_map: HashMap<FieldKey, ObjectStatePruneInfo>, // field_key -> ObjectStatePruneInfo hash
    // pruning_state_keys: Arc<RwLock<HashSet<FieldKey>>>,
    // pruning_state_root: H256,
    // state_roots: Arc<RwLock<BTreeMap<u64, StateRootInfo>>>, // timestamp -> StateRootInfo
}

impl StatePruner {
    pub fn new(moveos_store: MoveOSStore, latest_state_root: H256) -> Self {
        Self {
            moveos_store: Arc::new(moveos_store),
            latest_state_root,
            // latest_state_map: Arc::new(RwLock::new(HashMap::new())),
            latest_state_map: HashMap::new(),
            pruning_state_keys: Arc::new(RwLock::new(HashSet::new())),
            pruning_state_root: H256::zero(),
        }
    }

    // pub fn register_state_root(&self, state_root: H256, timestamp: u64) {
    //     self.state_roots.write().insert(timestamp, StateRootInfo {
    //         state_root,
    //         timestamp,
    //     });
    // }

    pub async fn start_pruning(&self) {
        loop {
            if let Err(e) = self.prune_states().await {
                warn!("Error pruning states: {}", e);
            }
            sleep(PRUNING_INTERVAL).await;
        }
    }

    async fn prune_states(&self) -> Result<()> {
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .saturating_sub(PRUNING_AGE_DAYS * 24 * 3600);

        // Get all state roots older than cutoff time
        let old_state_roots = self.get_old_state_roots(cutoff_time).await?;

        for state_root in old_state_roots {
            self.process_state_root(state_root).await?;
        }

        Ok(())
    }

    async fn get_old_state_roots(&self, cutoff_time: u64) -> Result<Vec<H256>> {
        let state_roots = self.state_roots.read();
        let old_roots: Vec<H256> = state_roots
            .range(..cutoff_time)
            .map(|(_, info)| info.state_root)
            .collect();
        
        Ok(old_roots)
    }

    async fn process_prune_state_root(&self, pruning_state_root: H256) -> Result<()> {
        let mut current_batch = Vec::new();
        let mut pruning_state_keys = HashSet::new();
        let mut count:u64 = 0;

        // Iterate through all fields in the state root
        let mut iter = self.moveos_store.state_store.iter(pruning_state_root, None)?;

        // let iter = moveos_store
        //     .get_state_store()
        //     .iter(obj_state_root, starting_key)?;
        //
        // let mut loop_time = Instant::now();
        for item in iter {
            let (k, v) = item?;
            if self.check_prune_state(&k, &v) {
                pruning_state_keys.insert(k.clone());
            }
            count += 1;
            // if count % 1_000_000 == 0 {
            //     println!(
            //         "exporting top_level_fields of object_id: {:?}({}), exported count: {}. cost: {:?}",
            //         object_id, object_name, count, loop_time.elapsed()
            //     );
            //     loop_time = Instant::now();
            // }
        }
        //
        // while let Some((field_key, object_state)) = iter.next() {
        //     let field_key = field_key?;
        //     let object_state = object_state?;
        //
        //     // Check if this field_key exists in latest_state_map
        //     let latest_hash = self.latest_state_map.read().get(&field_key).cloned();
        //
        //     if let Some(latest_hash) = latest_hash {
        //         // If the current object state hash matches latest, skip
        //         if latest_hash == object_state.merkle_hash() {
        //             continue;
        //         }
        //     }
        //
        //     // Add to pruning batch
        //     current_batch.push(field_key);
        //
        //     if current_batch.len() >= PRUNING_BATCH_SIZE {
        //         self.prune_batch(&current_batch).await?;
        //         current_batch.clear();
        //     }
        // }
        for (idx, item) in pruning_state_keys.into_iter().enumerate() {
            // Add to pruning batch
            current_batch.push(item);

            if current_batch.len() >= PRUNING_BATCH_SIZE {
                self.prune_batch(pruning_state_root, current_batch).await?;
                current_batch.clear();
            }
        }
        
        // Prune remaining batch
        if !current_batch.is_empty() {
            self.prune_batch(pruning_state_root, current_batch).await?;
        }
        
        Ok(())
    }

    fn check_prune_state(&self, prune_key: &FieldKey, prune_object_state: &ObjectState) -> Result<bool> {
        let now_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        let prune_time = prune_object_state.metadata.updated_at / 1000 + PRUNING_TIME_THRESHOLD;
        // Only clean up statedb older than 1 month
        if now_time >= prune_time {
            let latest_state_prune_info_opt = self.get_latest_state_prune_info(prune_key)?;
            if let Some(latest_state_prune_info) = latest_state_prune_info_opt {
                let prune_state_hash = prune_object_state.to_prune_hash()?;
                if prune_state_hash != latest_state_prune_info.prune_hash {
                    return Ok(true);
                } else {
                    // if prune_hash is the same, need to prune the state ?
                    return Ok(false);
                }
            } else {
                return Ok(true)
            }
        }

        Ok(false)
    }

    // export top level fields of an object, no recursive export child field
    pub async fn export_top_level_fields(
        moveos_store: &MoveOSStore,
        obj_state_root: H256,
        object_id: ObjectID,
        object_name: Option<String>, // human-readable object name for debug
        writer: &mut ExportWriter,
    ) -> Result<()> {
        let start_time = Instant::now();

        let starting_key = None;
        let mut count: u64 = 0;

        let object_name =
            object_name.unwrap_or(ExportObjectName::from_object_id(object_id.clone()).to_string());

        let iter = moveos_store
            .get_state_store()
            .iter(obj_state_root, starting_key)?;

        let mut loop_time = Instant::now();
        for item in iter {
            let (k, v) = item?;
            writer.write_record(&k, &v)?;
            count += 1;
            if count % 1_000_000 == 0 {
                println!(
                    "exporting top_level_fields of object_id: {:?}({}), exported count: {}. cost: {:?}",
                    object_id, object_name, count, loop_time.elapsed()
                );
                loop_time = Instant::now();
            }
        }

        println!(
            "Done. export_top_level_fields of object_id: {:?}({}), state_root: {:?}, exported count: {}. cost: {:?}",
            object_id,
            object_name,
            obj_state_root,
            count,
            start_time.elapsed()
        );
        Ok(())
    }

    fn get_latest_state_prune_info(&self, field_key: &FieldKey) -> Result<Option<ObjectStatePruneInfo>> {
        Ok(self.latest_state_map.get(field_key).cloned())
    }

    async fn prune_batch(&self, pruning_state_root: H256, field_keys: Vec<FieldKey>) -> Result<()> {
        let mut nodes_to_remove = BTreeMap::new();
        
        // Get all nodes that need to be removed
        for field_key in field_keys {
            if let Some(hash) = self.pruning_state_map.read().get(field_key) {
                nodes_to_remove.insert(*hash, vec![]);
            }
        }
        
        // Remove nodes from storage
        if !nodes_to_remove.is_empty() {
            self.node_store.remove_nodes(nodes_to_remove.keys().cloned().collect())?;
        }
        
        // Update pruning state map
        let mut pruning_map = self.pruning_state_map.write();
        for field_key in field_keys {
            pruning_map.remove(field_key);
        }
        
        info!("Pruned batch of {} field keys", field_keys.len());
        Ok(())
    }

    pub fn update_latest_state(&self, field_key: FieldKey, object_state: &ObjectState) {
        let hash = object_state.merkle_hash();
        self.latest_state_map.write().insert(field_key, hash);
    }
} 