// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::schema::global_states;
use crate::schema::table_change_sets;
use crate::schema::table_states;
use crate::types::{IndexedGlobalState, IndexedTableChangeSet, IndexedTableState};
use diesel::prelude::*;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::TypeTag;
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_api::jsonrpc_types::TableChangeSetView;
use rooch_types::indexer::state::IndexerTableChangeSet;
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
    /// The T struct tag of the object
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub object_type: String,
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
            object_type: state.object_type,
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

        let state = IndexedGlobalState {
            object_id,
            owner,
            flag: self.flag as u8,
            value: self.value.clone(),
            object_type: self.object_type.clone(),
            key_type: self.key_type.clone(),
            size: self.size as u64,
            created_at: self.created_at as u64,
            updated_at: self.updated_at as u64,
        };
        Ok(state)
    }
}

#[derive(Clone, Debug, Queryable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = table_states)]
pub struct StoredTableState {
    /// A primary key represents composite key of (table_handle, key_hex)
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    /// The state table handle
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub table_handle: String,
    /// The hex of the table key
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub key_hex: String,
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

impl From<IndexedTableState> for StoredTableState {
    fn from(state: IndexedTableState) -> Self {
        let id = format!("{}{}", state.table_handle, state.key_hex);
        Self {
            id,
            table_handle: state.table_handle.to_string(),
            key_hex: state.key_hex,
            value: state.value,
            value_type: state.value_type.to_canonical_string(),
            created_at: state.created_at as i64,
            updated_at: state.updated_at as i64,
        }
    }
}

impl StoredTableState {
    pub fn try_into_indexer_table_state(&self) -> Result<IndexedTableState, anyhow::Error> {
        let table_handle = ObjectID::from_str(self.table_handle.as_str())?;
        let value_type = TypeTag::from_str(self.value_type.as_str())?;

        let state = IndexedTableState {
            id: self.id.clone(),
            table_handle,
            key_hex: self.key_hex.clone(),
            value: self.value.clone(),
            value_type,
            created_at: self.created_at as u64,
            updated_at: self.updated_at as u64,
        };
        Ok(state)
    }
}

#[derive(Clone, Debug, Queryable, Insertable, QueryableByName)]
#[diesel(table_name = table_change_sets)]
pub struct StoredTableChangeSet {
    /// The tx order of this transaction which produce the table change set
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub tx_order: i64,
    /// The table handle index in the tx
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub table_handle_index: i64,
    /// The table handle
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub table_handle: String,
    /// The table change set, json format
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub table_change_set: String,
    /// The tx executed timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub created_at: i64,
}

impl From<IndexedTableChangeSet> for StoredTableChangeSet {
    fn from(state_change_set: IndexedTableChangeSet) -> Self {
        Self {
            tx_order: state_change_set.tx_order as i64,
            table_handle_index: state_change_set.table_handle_index as i64,
            table_handle: state_change_set.table_handle.to_string(),
            table_change_set: state_change_set.table_change_set,
            created_at: state_change_set.created_at as i64,
        }
    }
}

impl StoredTableChangeSet {
    pub fn try_into_indexer_state_change_set(
        &self,
    ) -> Result<IndexerTableChangeSet, anyhow::Error> {
        let table_handle = ObjectID::from_str(self.table_handle.as_str())?;
        let table_change_set: TableChangeSetView =
            serde_json::from_str(self.table_change_set.as_str())?;

        let indexer_state_change_set = IndexerTableChangeSet {
            tx_order: self.tx_order as u64,
            table_handle_index: self.table_handle_index as u64,
            table_handle,
            table_change_set: table_change_set.into(),
            created_at: self.created_at as u64,
        };
        Ok(indexer_state_change_set)
    }
}
