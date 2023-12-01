// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::indexer::Filter;
use anyhow::Result;
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
    pub table_handle_index: u64,
    pub table_handle: ObjectID,
    pub table_change_set: TableChangeSet,
    pub created_at: u64,
}

#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
pub struct IndexerStateID {
    pub tx_order: u64,
    pub table_handle_index: u64,
}

impl std::fmt::Display for IndexerStateID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IndexerStateID[tx order: {:?}, table handle  index: {}]",
            self.tx_order, self.table_handle_index,
        )
    }
}

impl IndexerStateID {
    pub fn new(tx_order: u64, table_handle_index: u64) -> Self {
        IndexerStateID {
            tx_order,
            table_handle_index,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StateFilter {
    /// Query by table handle.
    TableHandle(ObjectID),
}

impl StateFilter {
    fn try_matches(&self, item: &IndexerTableChangeSet) -> Result<bool> {
        Ok(match self {
            StateFilter::TableHandle(table_handle) => table_handle == &item.table_handle,
        })
    }
}

impl Filter<IndexerTableChangeSet> for StateFilter {
    fn matches(&self, item: &IndexerTableChangeSet) -> bool {
        self.try_matches(item).unwrap_or_default()
    }
}
