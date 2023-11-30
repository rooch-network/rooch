// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::schema::global_states;
use crate::schema::leaf_states;
use crate::schema::state_change_sets;
use crate::types::{IndexedGlobalState, IndexedLeafState, IndexedStateChangeSet};
use diesel::prelude::*;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::TypeTag;
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_api::jsonrpc_types::StateChangeSetView;
use rooch_types::indexer::state::IndexerStateChangeSet;
use std::str::FromStr;

#[derive(Clone, Debug, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = global_states)]
pub struct StoredGlobalState {
    /// The global state key
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub object_id: String,
    /// The owner of the object
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub owner: String,
    /// A flag to indicate whether the object is shared or frozen
    #[diesel(sql_type = diesel::sql_types::SmallInt)]
    pub flag: i16,
    /// The key type tag of the table
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub key_type: String,
    /// The value of the object, json format
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub value: String,
    /// The table length
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub size: i64,
    /// The object created timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub created_at: i64,
    /// The object updated timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub updated_at: i64,
}

impl From<IndexedGlobalState> for StoredGlobalState {
    fn from(state: IndexedGlobalState) -> Self {
        Self {
            object_id: state.object_id.to_string(),
            owner: state.owner.to_hex_literal(),
            flag: state.flag as i16,
            value: state.value,
            key_type: state.key_type,
            size: state.size as i64,
            created_at: state.created_at as i64,
            updated_at: state.updated_at as i64,
        }
    }
}

impl StoredGlobalState {
    pub fn try_into_indexer_global_state(&self) -> Result<IndexedGlobalState, anyhow::Error> {
        let object_id = ObjectID::from_str(self.object_id.as_str())?;
        let owner = AccountAddress::from_hex_literal(self.owner.as_str())?;
        // let key_type = TypeTag::from_str(self.key_type.as_str())?;

        let state = IndexedGlobalState {
            object_id,
            owner,
            flag: self.flag as u8,
            value: self.value.clone(),
            key_type: self.key_type.clone(),
            size: self.size as u64,
            created_at: self.created_at as u64,
            updated_at: self.updated_at as u64,
        };
        Ok(state)
    }
}

#[derive(Clone, Debug, Queryable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = leaf_states)]
pub struct StoredLeafState {
    /// A primary key represents composite key of (object_id, key_str)
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    /// The leaf state table handle
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub object_id: String,
    /// The hex of the table key
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub key_str: String,
    /// The value of the table, json format
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub value: String,
    /// The type tag of the value
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub value_type: String,
    /// The table item created timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub created_at: i64,
    /// The table item updated timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub updated_at: i64,
}

impl From<IndexedLeafState> for StoredLeafState {
    fn from(state: IndexedLeafState) -> Self {
        let id = format!("{}{}", state.object_id, state.key_str);
        Self {
            id,
            object_id: state.object_id.to_string(),
            key_str: state.key_str,
            value: state.value,
            value_type: state.value_type.to_canonical_string(),
            created_at: state.created_at as i64,
            updated_at: state.updated_at as i64,
        }
    }
}

impl StoredLeafState {
    pub fn try_into_indexer_leaf_state(&self) -> Result<IndexedLeafState, anyhow::Error> {
        let object_id = ObjectID::from_str(self.object_id.as_str())?;
        let value_type = TypeTag::from_str(self.value_type.as_str())?;

        let state = IndexedLeafState {
            id: self.id.clone(),
            object_id,
            key_str: self.key_str.clone(),
            value: self.value.clone(),
            value_type,
            created_at: self.created_at as u64,
            updated_at: self.updated_at as u64,
        };
        Ok(state)
    }
}

#[derive(Clone, Debug, Queryable, Insertable, QueryableByName)]
#[diesel(table_name = state_change_sets)]
pub struct StoredStateChangeSet {
    /// The tx order of this transaction which produce the state change set
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub tx_order: i64,
    /// The state change set, json format
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub state_change_set: String,
    /// The tx executed timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub created_at: i64,
}

impl From<IndexedStateChangeSet> for StoredStateChangeSet {
    fn from(state_change_set: IndexedStateChangeSet) -> Self {
        Self {
            tx_order: state_change_set.tx_order as i64,
            state_change_set: state_change_set.state_change_set,
            created_at: state_change_set.created_at as i64,
        }
    }
}

impl StoredStateChangeSet {
    pub fn try_into_indexer_state_change_set(
        &self,
    ) -> Result<IndexerStateChangeSet, anyhow::Error> {
        let state_change_set: StateChangeSetView =
            serde_json::from_str(self.state_change_set.as_str())?;

        let indexer_state_change_set = IndexerStateChangeSet {
            tx_order: self.tx_order as u64,
            state_change_set: state_change_set.into(),
            created_at: self.created_at as u64,
        };
        Ok(indexer_state_change_set)
    }
}
