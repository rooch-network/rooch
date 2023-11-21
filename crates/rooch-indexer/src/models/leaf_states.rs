// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::types::IndexedLeafState;
use diesel::prelude::*;
use move_core_types::language_storage::TypeTag;
use moveos_types::moveos_std::object::ObjectID;
use std::str::FromStr;

#[derive(Queryable, QueryableByName, Insertable, Debug, Clone)]
#[diesel(table_name = leaf_states)]
pub struct StoredLeafState {
    /// The leaf state table handle
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub object_id: String,
    /// The hash of the table key
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub key_hash: String,
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
        Self {
            object_id: state.object_id.to_string(),
            key_hash: state.key_hash,
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
            object_id,
            key_hash: self.key_hash.clone(),
            value: self.value.clone(),
            value_type,
            created_at: self.created_at as u64,
            updated_at: self.updated_at as u64,
        };
        Ok(state)
    }
}
