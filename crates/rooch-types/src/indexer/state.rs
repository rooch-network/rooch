// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::bitcoin::ord::Inscription;
use crate::bitcoin::utxo::UTXO;
use crate::indexer::Filter;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::Op;
use move_core_types::language_storage::{StructTag, TypeTag};
use moveos_types::move_types::type_tag_match;
use moveos_types::moveos_std::object::{is_dynamic_field_type, ObjectID, ObjectMeta};
use moveos_types::state::{MoveStructType, MoveType, ObjectChange, StateChangeSet};
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub static UTXO_TYPE_TAG: Lazy<TypeTag> = Lazy::new(UTXO::type_tag);

pub static INSCRIPTION_TYPE_TAG: Lazy<TypeTag> = Lazy::new(Inscription::type_tag);

/// Index all Object state, include child object
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectStateType {
    ObjectState, //all object states exclude utxo and inscription
    UTXO,
    Inscription,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct IndexerObjectStateChangeSet {
    pub object_states: IndexerObjectStateChanges,
    pub object_state_utxos: IndexerObjectStateChanges,
    pub object_state_inscriptions: IndexerObjectStateChanges,
}

impl IndexerObjectStateChangeSet {
    pub fn update_object_states(&mut self, state: IndexerObjectState) {
        if type_tag_match(&state.metadata.object_type, &UTXO_TYPE_TAG) {
            self.object_state_utxos.update_object_states.push(state)
        } else if type_tag_match(&state.metadata.object_type, &INSCRIPTION_TYPE_TAG) {
            self.object_state_inscriptions
                .update_object_states
                .push(state)
        } else {
            self.object_states.update_object_states.push(state)
        }
    }

    pub fn new_object_states(&mut self, state: IndexerObjectState) {
        if type_tag_match(&state.metadata.object_type, &UTXO_TYPE_TAG) {
            self.object_state_utxos.new_object_states.push(state)
        } else if type_tag_match(&state.metadata.object_type, &INSCRIPTION_TYPE_TAG) {
            self.object_state_inscriptions.new_object_states.push(state)
        } else {
            self.object_states.new_object_states.push(state)
        }
    }

    pub fn remove_object_states(&mut self, object_id: ObjectID, object_type: &TypeTag) {
        if type_tag_match(object_type, &UTXO_TYPE_TAG) {
            self.object_state_utxos
                .remove_object_states
                .push(object_id.to_string())
        } else if type_tag_match(object_type, &INSCRIPTION_TYPE_TAG) {
            self.object_state_inscriptions
                .remove_object_states
                .push(object_id.to_string())
        } else {
            self.object_states
                .remove_object_states
                .push(object_id.to_string())
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct IndexerObjectStateChanges {
    pub new_object_states: Vec<IndexerObjectState>,
    pub update_object_states: Vec<IndexerObjectState>,
    pub remove_object_states: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct IndexerObjectStatesIndexGenerator {
    pub object_states_index_generator: u64,
    pub object_state_utxos_index_generator: u64,
    pub object_state_inscriptions_generator: u64,
}

impl IndexerObjectStatesIndexGenerator {
    pub fn incr(&mut self, object_type: &TypeTag) {
        if type_tag_match(object_type, &UTXO_TYPE_TAG) {
            self.object_state_utxos_index_generator += 1;
        } else if type_tag_match(object_type, &INSCRIPTION_TYPE_TAG) {
            self.object_state_inscriptions_generator += 1;
        } else {
            self.object_states_index_generator += 1;
        }
    }

    pub fn get(&mut self, object_type: &TypeTag) -> u64 {
        if type_tag_match(object_type, &UTXO_TYPE_TAG) {
            self.object_state_utxos_index_generator
        } else if type_tag_match(object_type, &INSCRIPTION_TYPE_TAG) {
            self.object_state_inscriptions_generator
        } else {
            self.object_states_index_generator
        }
    }
}

pub fn handle_object_change(
    state_index_generator: &mut IndexerObjectStatesIndexGenerator,
    tx_order: u64,
    indexer_object_state_change_set: &mut IndexerObjectStateChangeSet,
    object_change: ObjectChange,
) -> Result<()> {
    let ObjectChange {
        metadata,
        value,
        fields,
    } = object_change;
    let object_id = metadata.id.clone();
    let object_type = metadata.object_type.clone();
    let state_index = state_index_generator.get(&object_type);

    // Do not index dynamic field object
    if is_dynamic_field_type(&object_type) {
        return Ok(());
    }
    if let Some(op) = value {
        match op {
            Op::Modify(_value) => {
                let state = IndexerObjectState::new(metadata.clone(), tx_order, state_index);
                indexer_object_state_change_set.update_object_states(state);
            }
            Op::Delete => {
                indexer_object_state_change_set.remove_object_states(object_id, &object_type);
            }
            Op::New(_value) => {
                let state = IndexerObjectState::new(metadata.clone(), tx_order, state_index);
                indexer_object_state_change_set.new_object_states(state);
            }
        }
    } else {
        //If value is not changed, we should update the metadata.
        let state = IndexerObjectState::new(metadata.clone(), tx_order, state_index);
        indexer_object_state_change_set.update_object_states(state);
    }

    state_index_generator.incr(&object_type);
    for (_key, change) in fields {
        handle_object_change(
            state_index_generator,
            tx_order,
            indexer_object_state_change_set,
            change,
        )?;
    }
    Ok(())
}

pub fn handle_revert_object_change(
    state_index_generator: &mut IndexerObjectStatesIndexGenerator,
    tx_order: u64,
    indexer_object_state_change_set: &mut IndexerObjectStateChangeSet,
    object_change: ObjectChange,
    object_mapping: &HashMap<ObjectID, ObjectMeta>,
) -> Result<()> {
    let ObjectChange {
        metadata,
        value,
        fields,
    } = object_change;
    let object_id = metadata.id.clone();
    let object_type = metadata.object_type.clone();
    let state_index = state_index_generator.get(&object_type);

    // Do not index dynamic field object
    if is_dynamic_field_type(&object_type) {
        return Ok(());
    }
    if let Some(op) = value {
        match op {
            Op::Modify(_value) => {
                // Keep the tx_order and state index consistent before reverting
                if let Some(previous_object_meta) = object_mapping.get(&object_id) {
                    let state = IndexerObjectState::new(
                        previous_object_meta.clone(),
                        tx_order,
                        state_index,
                    );
                    indexer_object_state_change_set.update_object_states(state);
                }
            }
            Op::Delete => {
                // Use the reverted tx_order and state index as the deleted restored tx_order and tx_order
                if let Some(previous_object_meta) = object_mapping.get(&object_id) {
                    let state = IndexerObjectState::new(
                        previous_object_meta.clone(),
                        tx_order,
                        state_index,
                    );
                    indexer_object_state_change_set.new_object_states(state);
                }
            }
            Op::New(_value) => {
                indexer_object_state_change_set.remove_object_states(object_id, &object_type);
            }
        }
    }

    state_index_generator.incr(&object_type);
    for (_key, change) in fields {
        handle_revert_object_change(
            state_index_generator,
            tx_order,
            indexer_object_state_change_set,
            change,
            object_mapping,
        )?;
    }
    Ok(())
}

pub fn collect_revert_object_change_ids(
    object_change: ObjectChange,
    object_ids: &mut Vec<ObjectID>,
) -> Result<()> {
    let ObjectChange {
        metadata,
        value,
        fields,
    } = object_change;
    let object_id = metadata.id.clone();
    let object_type = metadata.object_type.clone();

    // Do not index dynamic field object
    if is_dynamic_field_type(&object_type) {
        return Ok(());
    }
    if let Some(op) = value {
        match op {
            Op::Modify(_value) => {
                object_ids.push(object_id);
            }
            Op::Delete => {
                object_ids.push(object_id);
            }
            Op::New(_value) => {}
        }
    }

    for (_key, change) in fields {
        collect_revert_object_change_ids(change, object_ids)?;
    }
    Ok(())
}

#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema, Default,
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
        owner: AccountAddress,
        filter_out: bool,
    },
    /// Query by object type.
    ObjectType(StructTag),
    /// Query by owner.
    Owner(AccountAddress),
    /// Query by object ids.
    ObjectId(Vec<ObjectID>),
}

impl ObjectStateFilter {
    fn try_matches(&self, item: &IndexerObjectState) -> Result<bool> {
        Ok(match self {
            ObjectStateFilter::ObjectTypeWithOwner {
                object_type, owner, ..
            } => object_type == item.object_struct_tag() && owner == &item.metadata.owner,
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

#[derive(Clone, Debug)]
pub struct IndexerStateChangeSet {
    pub tx_order: u64,
    pub state_change_set: StateChangeSet,
    pub created_at: u64,
}
