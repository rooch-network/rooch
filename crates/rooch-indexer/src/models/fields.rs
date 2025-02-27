// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::schema::fields;
use diesel::prelude::*;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::TypeTag;
use moveos_types::moveos_std::object::{ObjectID, ObjectMeta};
use moveos_types::state::FieldKey;
use rooch_types::indexer::field::IndexerField;
use std::str::FromStr;

#[derive(Queryable, QueryableByName, Insertable, Debug, Clone)]
#[diesel(table_name = fields)]
pub struct StoredField {
    /// The object id
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    /// The parent object id
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub parent_id: String,
    /// The field key, it is a hash of (key|key_type)
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub field_key: String,
    // /// The origin key, string format
    // #[diesel(sql_type = diesel::sql_types::Text)]
    // pub name: String,
    /// The sort key, must be number format
    /// Since SQLite doesn't have a native i128 or i256 type,
    /// it needs to create a custom type to express larger values, such as coin value
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub sort_key: i64,
    /// The table item created timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub created_at: i64,
    /// The table item updated timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub updated_at: i64,
}

impl From<IndexerField> for StoredField {
    fn from(field: IndexerField) -> Self {
        Self {
            id: field.metadata.id.to_string(),
            parent_id: field
                .metadata
                .id
                .parent()
                .unwrap_or(ObjectID::root())
                .to_string(),
            field_key: field.field_key.to_hex_literal(),
            sort_key: field.sort_key as i64,
            created_at: field.metadata.created_at as i64,
            updated_at: field.metadata.updated_at as i64,
        }
    }
}

impl StoredField {
    pub fn try_into_indexer_field(&self) -> Result<IndexerField, anyhow::Error> {
        let id = ObjectID::from_str(self.id.as_str())?;
        let field_key = FieldKey::from_hex_literal(self.field_key.as_str())?;

        let metadata = ObjectMeta {
            id,
            owner: AccountAddress::ZERO, //default
            flag: 0,                     //default 0
            state_root: None,            //default
            size: 0,                     //default 0
            created_at: self.created_at as u64,
            updated_at: self.updated_at as u64,
            object_type: TypeTag::Address, //default
        };
        let table = IndexerField {
            field_key,
            metadata,
            sort_key: self.sort_key as u64,
        };
        Ok(table)
    }
}
