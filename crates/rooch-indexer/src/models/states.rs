// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::schema::object_states;
use diesel::prelude::*;
use move_core_types::language_storage::StructTag;
use moveos_types::h256::H256;
use moveos_types::moveos_std::object::ObjectID;
use rooch_types::address::RoochAddress;
use rooch_types::indexer::state::IndexerObjectState;
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
            object_type: state.object_type.to_string(),
            state_root: format!("{:?}", state.state_root),
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
        let owner = RoochAddress::from_str(self.owner.as_str())?;
        let object_type = StructTag::from_str(self.object_type.as_str())?;
        let state_root = H256::from_str(self.state_root.as_str())?;

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
