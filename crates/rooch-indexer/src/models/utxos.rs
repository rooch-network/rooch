// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use diesel::prelude::*;
use move_core_types::account_address::AccountAddress;
use moveos_types::moveos_std::object::{ObjectID, ObjectMeta};
use moveos_types::state::MoveType;
use rooch_types::bitcoin::utxo::UTXO;
use rooch_types::indexer::state::IndexerObjectState;
use std::str::FromStr;

#[derive(Queryable, QueryableByName, Insertable, Debug, Clone)]
#[diesel(table_name = utxos)]
pub struct StoredUTXO {
    /// The global state key
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    /// The owner of the object
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub owner: String,
    // /// A flag to indicate whether the object is shared or frozen
    // #[diesel(sql_type = diesel::sql_types::SmallInt)]
    // pub flag: i16,
    // /// The table state root of the object
    // #[diesel(sql_type = diesel::sql_types::Text)]
    // pub state_root: String,
    // /// The table length
    // #[diesel(sql_type = diesel::sql_types::BigInt)]
    // pub size: i64,
    /// The object created timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub created_at: i64,
    /// The object updated timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub updated_at: i64,
    // /// The T struct tag of the object value
    // #[diesel(sql_type = diesel::sql_types::Text)]
    // pub object_type: String,
    /// The tx order of this transaction
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub tx_order: i64,
    /// The state index in the tx
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub state_index: i64,
}

impl From<IndexerObjectState> for StoredUTXO {
    fn from(state: IndexerObjectState) -> Self {
        let metadata = state.metadata;
        let tx_order = state.tx_order;
        let state_index = state.state_index;
        Self {
            id: metadata.id.to_string(),
            owner: metadata.owner.to_hex_literal(),
            // flag: metadata.flag as i16,
            // state_root: metadata
            //     .state_root
            //     .map(|h| format!("{:?}", h))
            //     .unwrap_or_default(),
            // size: metadata.size as i64,
            created_at: metadata.created_at as i64,
            updated_at: metadata.updated_at as i64,
            // object_type: metadata.object_type.to_string(),
            tx_order: tx_order as i64,
            state_index: state_index as i64,
        }
    }
}

impl StoredUTXO {
    pub fn try_parse_indexer_object_state(&self) -> Result<IndexerObjectState, anyhow::Error> {
        let id = ObjectID::from_str(self.id.as_str())?;
        let owner = AccountAddress::from_str(self.owner.as_str())?;
        // let object_type = TypeTag::from_str(self.object_type.as_str())?;
        let object_type = UTXO::type_tag();
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

    // pub fn try_parse_id(&self) -> Result<(ObjectID, IndexerStateID), anyhow::Error> {
    //     let tx_order = self.tx_order as u64;
    //     let state_index = self.state_index as u64;
    //     let indexer_state_id = IndexerStateID {
    //         tx_order,
    //         state_index,
    //     };
    //     Ok((ObjectID::from_str(self.id.as_str())?, indexer_state_id))
    // }
}
