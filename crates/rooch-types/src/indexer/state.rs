// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::indexer::Filter;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::{StructTag, TypeTag};
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::{StateChangeSet, TableChangeSet};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct IndexerStateChangeSet {
    pub tx_order: u64,
    pub state_change_set: StateChangeSet,
    pub created_at: u64,
}

#[derive(Clone, Debug)]
pub struct IndexerTableChangeSet {
    pub tx_order: u64,
    pub state_index: u64,
    pub table_handle: ObjectID,
    pub table_change_set: TableChangeSet,
    pub created_at: u64,
}

#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
pub struct IndexerStateID {
    pub tx_order: u64,
    pub state_index: u64,
}

impl std::fmt::Display for IndexerStateID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IndexerStateID[tx order: {:?}, state index: {}]",
            self.tx_order, self.state_index,
        )
    }
}

impl IndexerStateID {
    pub fn new(tx_order: u64, state_index: u64) -> Self {
        IndexerStateID {
            tx_order,
            state_index,
        }
    }
}

#[derive(Clone, Debug)]
pub struct IndexerGlobalState {
    pub object_id: ObjectID,
    pub owner: AccountAddress,
    pub flag: u8,
    pub value: String,
    pub object_type: StructTag,
    pub key_type: String,
    pub size: u64,
    pub tx_order: u64,
    pub state_index: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug)]
pub struct IndexerTableState {
    pub table_handle: ObjectID,
    pub key_hex: String,
    pub value: String,
    pub value_type: TypeTag,
    pub tx_order: u64,
    pub state_index: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GlobalStateFilter {
    /// Query by object type and owner.
    ObjectTypeWithOwner {
        object_type: StructTag,
        owner: AccountAddress,
    },
    /// Query by object type.
    ObjectType(StructTag),
    /// Query by owner.
    Owner(AccountAddress),
    /// Query by object id.
    ObjectId(ObjectID),
}

impl GlobalStateFilter {
    fn try_matches(&self, item: &IndexerGlobalState) -> Result<bool> {
        Ok(match self {
            GlobalStateFilter::ObjectTypeWithOwner { object_type, owner } => {
                object_type == &item.object_type && owner == &item.owner
            }
            GlobalStateFilter::ObjectType(object_type) => object_type == &item.object_type,
            GlobalStateFilter::Owner(owner) => owner == &item.owner,
            GlobalStateFilter::ObjectId(object_id) => object_id == &item.object_id,
        })
    }
}

impl Filter<IndexerGlobalState> for GlobalStateFilter {
    fn matches(&self, item: &IndexerGlobalState) -> bool {
        self.try_matches(item).unwrap_or_default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TableStateFilter {
    /// Query by table handle.
    TableHandle(ObjectID),
}

impl TableStateFilter {
    fn try_matches(&self, item: &IndexerTableState) -> Result<bool> {
        Ok(match self {
            TableStateFilter::TableHandle(table_handle) => table_handle == &item.table_handle,
        })
    }
}

impl Filter<IndexerTableState> for TableStateFilter {
    fn matches(&self, item: &IndexerTableState) -> bool {
        self.try_matches(item).unwrap_or_default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StateSyncFilter {
    /// Query by table handle.
    TableHandle(ObjectID),
}

impl StateSyncFilter {
    fn try_matches(&self, item: &IndexerTableChangeSet) -> Result<bool> {
        Ok(match self {
            StateSyncFilter::TableHandle(table_handle) => table_handle == &item.table_handle,
        })
    }
}

impl Filter<IndexerTableChangeSet> for StateSyncFilter {
    fn matches(&self, item: &IndexerTableChangeSet) -> bool {
        self.try_matches(item).unwrap_or_default()
    }
}
