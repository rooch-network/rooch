// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::schema::field_states;
use crate::schema::object_states;
use crate::utils;
use diesel::prelude::*;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::{StructTag, TypeTag};
use moveos_types::moveos_std::object::ObjectID;
use rooch_types::indexer::state::{IndexerFieldState, IndexerObjectState};
use std::str::FromStr;

#[derive(Queryable, QueryableByName, Insertable, Debug, Clone)]
#[diesel(table_name = object_states)]
pub struct StoredObjectState {
    /// The global state key
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub object_id: String,
    /// The owner of the object
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub owner: String,
    /// A flag to indicate whether the object is shared or frozen
    #[diesel(sql_type = diesel::sql_types::SmallInt)]
    pub flag: i16,
    /// The T struct tag of the object value
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub object_type: String,
    /// The table state root of the object
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub state_root: String,
    /// The table length
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub size: i64,
    /// The tx order of this transaction
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub tx_order: i64,
    /// The state index in the tx
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub state_index: i64,
    /// The object created timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub created_at: i64,
    /// The object updated timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub updated_at: i64,
}

impl From<IndexerObjectState> for StoredObjectState {
    fn from(state: IndexerObjectState) -> Self {
        Self {
            object_id: state.object_id.to_string(),
            owner: state.owner.to_hex_literal(),
            flag: state.flag as i16,
            object_type: utils::format_struct_tag(&state.object_type),
            state_root: state.state_root.to_hex_literal(),
            size: state.size as i64,
            tx_order: state.tx_order as i64,
            state_index: state.state_index as i64,
            created_at: state.created_at as i64,
            updated_at: state.updated_at as i64,
        }
    }
}

impl StoredObjectState {
    pub fn try_into_indexer_global_state(&self) -> Result<IndexerObjectState, anyhow::Error> {
        let object_id = ObjectID::from_str(self.object_id.as_str())?;
        let owner = AccountAddress::from_hex_literal(self.owner.as_str())?;

        let object_type = StructTag::from_str(self.object_type.as_str())?;
        let state_root = AccountAddress::from_hex_literal(self.state_root.as_str())?;

        let state = IndexerObjectState {
            object_id,
            owner,
            flag: self.flag as u8,
            object_type,
            state_root,
            size: self.size as u64,
            tx_order: self.tx_order as u64,
            state_index: self.state_index as u64,
            created_at: self.created_at as u64,
            updated_at: self.updated_at as u64,
        };
        Ok(state)
    }
}

#[derive(Queryable, QueryableByName, Insertable, Debug, Clone)]
#[diesel(table_name = field_states)]
pub struct StoredFieldState {
    /// The state table handle
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub object_id: String,
    /// The hex of the table key
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub key_hex: String,
    /// The type tag of the key
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub key_type: String,
    /// The type tag of the value
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub value_type: String,
    /// The tx order of this transaction
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub tx_order: i64,
    /// The state index in the tx
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub state_index: i64,
    /// The table item created timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub created_at: i64,
    /// The table item updated timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub updated_at: i64,
}

impl From<IndexerFieldState> for StoredFieldState {
    fn from(state: IndexerFieldState) -> Self {
        Self {
            object_id: state.object_id.to_string(),
            key_hex: state.key_hex,
            key_type: state.key_type.to_string(),
            value_type: state.value_type.to_string(),
            tx_order: state.tx_order as i64,
            state_index: state.state_index as i64,
            created_at: state.created_at as i64,
            updated_at: state.updated_at as i64,
        }
    }
}

impl StoredFieldState {
    pub fn try_into_indexer_table_state(&self) -> Result<IndexerFieldState, anyhow::Error> {
        let object_id = ObjectID::from_str(self.object_id.as_str())?;
        let key_type = TypeTag::from_str(self.key_type.as_str())?;
        let value_type = TypeTag::from_str(self.value_type.as_str())?;

        let state = IndexerFieldState {
            object_id,
            key_hex: self.key_hex.clone(),
            key_type,
            value_type,
            tx_order: self.tx_order as u64,
            state_index: self.state_index as u64,
            created_at: self.created_at as u64,
            updated_at: self.updated_at as u64,
        };
        Ok(state)
    }
}
