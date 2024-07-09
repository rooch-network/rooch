// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::RoochAddress;
use crate::bitcoin::utxo::UTXO;
use crate::indexer::Filter;
use anyhow::Result;
use move_core_types::effects::Op;
use move_core_types::language_storage::StructTag;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::{MoveStructType, ObjectChange, ObjectState, StateChangeSet};
use moveos_types::state_resolver::StateResolver;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Index all Object state, include child object
#[derive(Debug, Clone)]
pub struct IndexerObjectState {
    // The global state key
    pub object_id: ObjectID,
    // The owner of the object
    pub owner: RoochAddress,
    // A flag to indicate whether the object is shared or frozen
    pub flag: u8,
    // The table state root of the object
    pub state_root: Option<H256>,
    // The table length
    pub size: u64,
    // The T struct tag of the object value
    pub object_type: StructTag,
    // The tx order of this transaction
    pub tx_order: u64,
    // The state index in the tx
    pub state_index: u64,
    // The object created timestamp on chain
    pub created_at: u64,
    // The object updated timestamp on chain
    pub updated_at: u64,
}

impl IndexerObjectState {
    pub fn new_from_object_state(state: ObjectState, tx_order: u64, state_index: u64) -> Self {
        let (metadata, _value) = state.into_inner();
        let object_type = metadata.value_struct_tag().clone();
        IndexerObjectState {
            object_id: metadata.id,
            owner: metadata.owner.into(),
            flag: metadata.flag,
            state_root: metadata.state_root,
            size: metadata.size,
            object_type,
            tx_order,
            state_index,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
        }
    }

    pub fn is_utxo_object_state(&self) -> bool {
        self.object_type == UTXO::struct_tag()
    }

    pub fn indexer_state_id(&self) -> IndexerStateID {
        IndexerStateID::new(self.tx_order, self.state_index)
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
    resolver: &dyn StateResolver,
) -> Result<u64> {
    let ObjectChange {
        metadata,
        value,
        fields,
    } = object_change;
    let object_id = metadata.id.clone();
    if let Some(op) = value {
        match op {
            Op::Modify(value) => {
                // refresh object to acquire lastest object state root
                //TODO we should update the state_root in the ObjectChange after apply.
                let refresh_object = resolver
                    .get_object(&object_id)?
                    .unwrap_or(ObjectState::new(metadata, value));

                let state = IndexerObjectState::new_from_object_state(
                    refresh_object,
                    tx_order,
                    state_index_generator,
                );
                indexer_object_state_changes
                    .update_object_states
                    .push(state);
            }
            Op::Delete => {
                indexer_object_state_changes
                    .remove_object_states
                    .push(object_id.to_string());
            }
            Op::New(value) => {
                // refresh object to acquire lastest object state root
                let refresh_object = resolver
                    .get_object(&object_id)?
                    .unwrap_or(ObjectState::new(metadata, value));
                let state = IndexerObjectState::new_from_object_state(
                    refresh_object,
                    tx_order,
                    state_index_generator,
                );
                indexer_object_state_changes.new_object_states.push(state);
            }
        }
    } else {
        //If value is not changed, we should update the metadata.
        let refresh_object = resolver
            .get_object(&object_id)?
            .ok_or_else(|| anyhow::anyhow!("Object {} not found for indexer", metadata.id))?;
        let state = IndexerObjectState::new_from_object_state(
            refresh_object,
            tx_order,
            state_index_generator,
        );
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
            resolver,
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
                object_type == &item.object_type && owner == &item.owner
            }
            ObjectStateFilter::ObjectType(object_type) => object_type == &item.object_type,
            ObjectStateFilter::Owner(owner) => owner == &item.owner,
            ObjectStateFilter::ObjectId(object_ids) => {
                object_ids.len() == 1 && object_ids[0] == item.object_id
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
