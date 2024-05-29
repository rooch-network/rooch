// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::RoochAddress;
use crate::bitcoin::utxo::UTXO;
use crate::indexer::Filter;
use anyhow::Result;
use move_core_types::effects::Op;
use move_core_types::language_storage::{StructTag, TypeTag};
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::{ObjectID, RawObject};
use moveos_types::state::{
    FieldChange, KeyState, MoveStructType, ObjectChange, State, StateChangeSet,
};
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
    pub state_root: H256,
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
    pub fn new_from_raw_object(raw_object: RawObject, tx_order: u64, state_index: u64) -> Self {
        IndexerObjectState {
            object_id: raw_object.id,
            owner: raw_object.owner.into(),
            flag: raw_object.flag,
            state_root: H256::from(raw_object.state_root.into_bytes()),
            size: raw_object.size,
            object_type: raw_object.value.struct_tag,
            tx_order,
            state_index,

            //TODO record transaction timestamp
            created_at: 0,
            updated_at: 0,
        }
    }

    pub fn is_utxo_object_state(&self) -> bool {
        self.object_type == UTXO::struct_tag()
    }

    pub fn try_new_from_state(
        tx_order: u64,
        state_index: u64,
        refresh_object: RawObject,
    ) -> Result<IndexerObjectState> {
        let state = IndexerObjectState::new_from_raw_object(refresh_object, tx_order, state_index);
        Ok(state)
    }

    pub fn indexer_state_id(&self) -> IndexerStateID {
        IndexerStateID::new(self.tx_order, self.state_index)
    }
}

/// Index all Object dynamic field
#[derive(Debug, Clone)]
pub struct IndexerFieldState {
    // The state object id
    pub object_id: ObjectID,
    // The hex of the field key state
    pub key_hex: String,
    // The type tag of the key
    pub key_type: TypeTag,
    // The type tag of the value
    pub value_type: TypeTag,
    // The tx order of this transaction
    pub tx_order: u64,
    // The state index in the tx
    pub state_index: u64,
    // The table item created timestamp on chain
    pub created_at: u64,
    // The table item updated timestamp on chain
    pub updated_at: u64,
}

impl IndexerFieldState {
    pub fn new(
        object_id: ObjectID,
        key_hex: String,
        key_type: TypeTag,
        value_type: TypeTag,
        tx_order: u64,
        state_index: u64,
    ) -> Self {
        IndexerFieldState {
            object_id,
            key_hex,
            key_type,
            value_type,
            tx_order,
            state_index,

            //TODO record transaction timestamp
            created_at: 0,
            updated_at: 0,
        }
    }

    pub fn new_field_state(
        key: KeyState,
        value: State,
        object_id: ObjectID,
        tx_order: u64,
        state_index: u64,
    ) -> IndexerFieldState {
        let key_hex = key.to_string();
        Self::new(
            object_id,
            key_hex,
            key.key_type,
            value.value_type,
            tx_order,
            state_index,
        )
    }
}
#[derive(Clone, Debug, Default)]
pub struct IndexerObjectStateChanges {
    pub new_object_states: Vec<IndexerObjectState>,
    pub update_object_states: Vec<IndexerObjectState>,
    pub remove_object_states: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct IndexerFieldStateChanges {
    pub new_field_states: Vec<IndexerFieldState>,
    pub update_field_states: Vec<IndexerFieldState>,
    // When remove table handle, first delete table handle from global states,
    // then delete all states which belongs to the object_id from table states
    pub remove_field_states: Vec<(String, String)>,
    pub remove_field_states_by_object_id: Vec<String>,
}

pub fn handle_object_change(
    mut state_index_generator: u64,
    tx_order: u64,
    indexer_object_state_changes: &mut IndexerObjectStateChanges,
    indexer_field_state_changes: &mut IndexerFieldStateChanges,
    object_id: ObjectID,
    object_change: ObjectChange,
    resolver: &dyn StateResolver,
) -> Result<u64> {
    let ObjectChange { op, fields } = object_change;
    // refresh object to acquire lastest object state root
    let refresh_object = resolver
        .get_object(&object_id)?
        .ok_or(anyhow::anyhow!("Object should exist"))?;

    if let Some(op) = op {
        match op {
            Op::Modify(value) => {
                debug_assert!(value.is_object());
                let state = IndexerObjectState::try_new_from_state(
                    tx_order,
                    state_index_generator,
                    refresh_object,
                )?;
                indexer_object_state_changes
                    .update_object_states
                    .push(state);
            }
            Op::Delete => {
                indexer_object_state_changes
                    .remove_object_states
                    .push(object_id.to_string());
                indexer_field_state_changes
                    .remove_field_states_by_object_id
                    .push(object_id.to_string());
            }
            Op::New(value) => {
                debug_assert!(value.is_object());
                let state = IndexerObjectState::try_new_from_state(
                    tx_order,
                    state_index_generator,
                    refresh_object,
                )?;
                indexer_object_state_changes.new_object_states.push(state);
            }
        }
    }

    state_index_generator += 1;
    for (key, change) in fields {
        match change {
            FieldChange::Normal(normal_change) => {
                match normal_change.op {
                    Op::Modify(value) => {
                        let state = IndexerFieldState::new_field_state(
                            key,
                            value,
                            object_id.clone(),
                            tx_order,
                            state_index_generator,
                        );
                        indexer_field_state_changes.update_field_states.push(state);
                    }
                    Op::Delete => {
                        indexer_field_state_changes
                            .remove_field_states
                            .push((object_id.to_string(), key.to_string()));
                    }
                    Op::New(value) => {
                        let state = IndexerFieldState::new_field_state(
                            key,
                            value,
                            object_id.clone(),
                            tx_order,
                            state_index_generator,
                        );
                        indexer_field_state_changes.new_field_states.push(state);
                    }
                }
                state_index_generator += 1;
            }
            FieldChange::Object(object_change) => {
                state_index_generator = handle_object_change(
                    state_index_generator,
                    tx_order,
                    indexer_object_state_changes,
                    indexer_field_state_changes,
                    key.as_object_id()?,
                    object_change,
                    resolver,
                )?;
            }
        }
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
pub enum FieldStateFilter {
    /// Query field by object id.
    ObjectId(ObjectID),
}

impl FieldStateFilter {
    fn try_matches(&self, item: &IndexerFieldState) -> Result<bool> {
        Ok(match self {
            FieldStateFilter::ObjectId(object_id) => object_id == &item.object_id,
        })
    }
}

impl Filter<IndexerFieldState> for FieldStateFilter {
    fn matches(&self, item: &IndexerFieldState) -> bool {
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
