// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::schema::inscriptions;
use diesel::prelude::*;
use move_core_types::account_address::AccountAddress;
use moveos_types::moveos_std::object::{ObjectID, ObjectMeta};
use moveos_types::state::MoveType;
use rooch_types::bitcoin::ord::Inscription;
use rooch_types::indexer::state::IndexerObjectState;
use std::str::FromStr;

#[derive(Queryable, QueryableByName, Insertable, Debug, Clone)]
#[diesel(table_name = inscriptions)]
pub struct StoredInscription {
    /// The global state key
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    /// The owner of the object
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub owner: String,
    /// The object created timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub created_at: i64,
    /// The object updated timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub updated_at: i64,
    /// The tx order of this transaction
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub tx_order: i64,
    /// The state index in the tx
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub state_index: i64,
}

impl From<IndexerObjectState> for StoredInscription {
    fn from(state: IndexerObjectState) -> Self {
        let metadata = state.metadata;
        let tx_order = state.tx_order;
        let state_index = state.state_index;
        Self {
            id: metadata.id.to_string(),
            owner: metadata.owner.to_hex_literal(),
            created_at: metadata.created_at as i64,
            updated_at: metadata.updated_at as i64,
            tx_order: tx_order as i64,
            state_index: state_index as i64,
        }
    }
}

impl StoredInscription {
    pub fn try_parse_indexer_object_state(&self) -> Result<IndexerObjectState, anyhow::Error> {
        let id = ObjectID::from_str(self.id.as_str())?;
        let owner = AccountAddress::from_str(self.owner.as_str())?;
        let object_type = Inscription::type_tag();
        let state_root = None;
        let metadata = ObjectMeta {
            id,
            owner,
            flag: 0, //default 0
            state_root,
            size: 0, //default 0
            created_at: self.created_at as u64,
            updated_at: self.updated_at as u64,
            object_type,
        };
        let state = IndexerObjectState {
            metadata,
            tx_order: self.tx_order as u64,
            state_index: self.state_index as u64,
        };
        Ok(state)
    }
}
