// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::schema::field_states;
use crate::schema::object_states;
use crate::schema::table_change_sets;
use crate::types::{IndexedFieldState, IndexedObjectState, IndexedTableChangeSet};
use diesel::prelude::*;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::{StructTag, TypeTag};
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_api::jsonrpc_types::TableChangeSetView;
use rooch_types::indexer::state::{IndexerFieldState, IndexerObjectState, IndexerTableChangeSet};
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
    /// The value of the object, json format
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub value: String,
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

impl From<IndexedObjectState> for StoredObjectState {
    fn from(state: IndexedObjectState) -> Self {
        Self {
            object_id: state.object_id.to_string(),
            owner: state.owner.to_hex_literal(),
            flag: state.flag as i16,
            value: state.value,
            object_type: state.object_type,
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
            value: self.value.clone(),
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
    /// The key of the table, json format
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub key_str: String,
    /// The value of the table, json format
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub value: String,
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

impl From<IndexedFieldState> for StoredFieldState {
    fn from(state: IndexedFieldState) -> Self {
        Self {
            object_id: state.object_id.to_string(),
            key_hex: state.key_hex,
            key_str: state.key_str,
            value: state.value,
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
            key_str: self.key_str.clone(),
            value: self.value.clone(),
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

#[derive(Clone, Debug, Queryable, Insertable, QueryableByName)]
#[diesel(table_name = table_change_sets)]
pub struct StoredTableChangeSet {
    /// The tx order of this transaction which produce the table change set
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub tx_order: i64,
    /// The table handle index in the tx
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub state_index: i64,
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
            state_index: state_change_set.state_index as i64,
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
            state_index: self.state_index as u64,
            table_handle,
            table_change_set: table_change_set.into(),
            created_at: self.created_at as u64,
        };
        Ok(indexer_state_change_set)
    }
}
