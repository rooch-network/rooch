// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::RoochAddress;
use crate::bitcoin::utxo::UTXO;
use crate::indexer::Filter;
use anyhow::Result;
use move_core_types::effects::Op;
use move_core_types::language_storage::StructTag;
use moveos_types::moveos_std::object::{ObjectID, ObjectMeta};
use moveos_types::state::{MoveStructType, ObjectChange, StateChangeSet};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Index all Object state, include child object
#[derive(Debug, Clone)]
pub struct IndexerObjectState {
    pub metadata: ObjectMeta,
    // The tx order of this transaction
    pub tx_order: u64,
    // The state index in the tx
    pub state_index: u64,
}

impl IndexerObjectState {
    pub fn new(metadata: ObjectMeta, tx_order: u64, state_index: u64) -> Self {
        IndexerObjectState {
            metadata,
            tx_order,
            state_index,
        }
    }

    pub fn is_utxo_object_state(&self) -> bool {
        self.metadata.match_struct_type(&UTXO::struct_tag())
    }

    pub fn indexer_state_id(&self) -> IndexerStateID {
        IndexerStateID::new(self.tx_order, self.state_index)
    }

    pub fn object_struct_tag(&self) -> &StructTag {
        self.metadata.object_struct_tag()
    }
}

#[derive(Clone, Debug, Default)]
pub struct IndexerObjectStateChanges {
    pub new_object_states: Vec<IndexerObjectState>,
    pub update_object_states: Vec<IndexerObjectState>,
    pub remove_object_states: Vec<String>,
}

pub fn handle_object_change(
    mut state_index_generator: u64,
    tx_order: u64,
    indexer_object_state_changes: &mut IndexerObjectStateChanges,
    object_change: ObjectChange,
) -> Result<u64> {
    let ObjectChange {
        metadata,
        value,
        fields,
    } = object_change;
    let object_id = metadata.id.clone();
    if let Some(op) = value {
        match op {
            Op::Modify(_value) => {
                let state = IndexerObjectState::new(metadata, tx_order, state_index_generator);
                indexer_object_state_changes
                    .update_object_states
                    .push(state);
            }
            Op::Delete => {
                indexer_object_state_changes
                    .remove_object_states
                    .push(object_id.to_string());
            }
            Op::New(_value) => {
                let state = IndexerObjectState::new(metadata, tx_order, state_index_generator);
                indexer_object_state_changes.new_object_states.push(state);
            }
        }
    } else {
        //If value is not changed, we should update the metadata.
        let state = IndexerObjectState::new(metadata, tx_order, state_index_generator);
        indexer_object_state_changes
            .update_object_states
            .push(state);
    }

    state_index_generator += 1;
    for (_key, change) in fields {
        state_index_generator = handle_object_change(
            state_index_generator,
            tx_order,
            indexer_object_state_changes,
            change,
        )?;
    }
    Ok(state_index_generator)
}

#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
pub struct IndexerStateID {
    pub tx_order: u64,
    pub state_index: u64,
}

impl std::fmt::Display for IndexerStateID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IndexerStateID[tx order: {:?}, state index: {}]",
            self.tx_order, self.state_index,
        )
    }
}

impl IndexerStateID {
    pub fn new(tx_order: u64, state_index: u64) -> Self {
        IndexerStateID {
            tx_order,
            state_index,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ObjectStateFilter {
    /// Query by object type and owner.
    ObjectTypeWithOwner {
        object_type: StructTag,
        owner: RoochAddress,
    },
    /// Query by object type.
    ObjectType(StructTag),
    /// Query by owner.
    Owner(RoochAddress),
    /// Query by object ids.
    ObjectId(Vec<ObjectID>),
}

impl ObjectStateFilter {
    fn try_matches(&self, item: &IndexerObjectState) -> Result<bool> {
        Ok(match self {
            ObjectStateFilter::ObjectTypeWithOwner { object_type, owner } => {
                object_type == item.object_struct_tag() && owner == &item.metadata.owner
            }
            ObjectStateFilter::ObjectType(object_type) => object_type == item.object_struct_tag(),
            ObjectStateFilter::Owner(owner) => owner == &item.metadata.owner,
            ObjectStateFilter::ObjectId(object_ids) => {
                object_ids.len() == 1 && object_ids[0] == item.metadata.id
            }
        })
    }
}

impl Filter<IndexerObjectState> for ObjectStateFilter {
    fn matches(&self, item: &IndexerObjectState) -> bool {
        self.try_matches(item).unwrap_or_default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StateSyncFilter {
    /// Query by object id.
    ObjectId(ObjectID),
}

#[derive(Clone, Debug)]
pub struct IndexerStateChangeSet {
    pub tx_order: u64,
    pub state_change_set: StateChangeSet,
    pub created_at: u64,
}
