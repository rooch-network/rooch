// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::schema::fields;
use diesel::prelude::*;
use moveos_types::moveos_std::object::ObjectID;
use rooch_types::indexer::field::IndexerField;
use std::str::FromStr;

#[derive(Queryable, QueryableByName, Insertable, Debug, Clone)]
#[diesel(table_name = fields)]
pub struct StoredField {
    /// The object id
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    /// The field key, it is a hash of (key|key_type)
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub field_key: String,
    /// The origin key, string format
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    /// The origin value, must be number format
    /// Since SQLite doesn't have a native i128 or i256 type,
    /// it needs to create a custom type to express larger values, such as coin value
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub val: i64,
    /// The table item created timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub created_at: i64,
    /// The table item updated timestamp on chain
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub updated_at: i64,
}

impl From<IndexerField> for StoredField {
    fn from(table: IndexerField) -> Self {
        Self {
            id: table.id.to_string(),
            field_key: table.field_key,
            name: table.name,
            val: table.value as i64,
            created_at: table.created_at as i64,
            updated_at: table.updated_at as i64,
        }
    }
}

impl StoredField {
    pub fn try_into_indexer_field(&self) -> Result<IndexerField, anyhow::Error> {
        let object_id = ObjectID::from_str(self.id.as_str())?;

        let table = IndexerField {
            id: object_id,
            field_key: self.field_key.clone(),
            name: self.name.clone(),
            value: self.val as u64,
            created_at: self.created_at as u64,
            updated_at: self.updated_at as u64,
        };
        Ok(table)
    }
}
