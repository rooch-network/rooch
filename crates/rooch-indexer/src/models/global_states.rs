// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::types::IndexedGlobalState;
use diesel::prelude::*;
use move_core_types::account_address::AccountAddress;
use moveos_types::moveos_std::object::ObjectID;
use std::str::FromStr;

#[derive(Queryable, QueryableByName, Insertable, Debug, Clone)]
#[diesel(table_name = global_states)]
pub struct StoredGlobalState {
    /// The global state table handle
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub object_id: String,
    /// The owner of the object
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub owner: String,
    /// A flag to indicate whether the object is shared or frozen
    #[diesel(sql_type = diesel::sql_types::SmallInt)]
    pub flag: i16,
    /// The value of the object, json format
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub value: String,
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
            created_at: state.created_at as i64,
            updated_at: state.updated_at as i64,
        }
    }
}

impl StoredGlobalState {
    pub fn try_into_indexer_global_state(&self) -> Result<IndexedGlobalState, anyhow::Error> {
        let object_id = ObjectID::from_str(self.object_id.as_str())?;
        let owner = AccountAddress::from_hex_literal(self.owner.as_str())?;

        let state = IndexedGlobalState {
            object_id,
            owner,
            flag: self.flag as u8,
            value: self.value.clone(),
            created_at: self.created_at as u64,
            updated_at: self.updated_at as u64,
        };
        Ok(state)
    }
}
