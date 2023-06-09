// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
/// Public types for tables. The raw table implementation in moveos-stdlib natives.
/// We put them in moveos-types to avoid circular dependencies.
use crate::{object::ObjectID, state::State};
use move_core_types::{effects::Op, language_storage::TypeTag};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

/// The representation of a table handle. This is created from truncating a sha3-256 based
/// hash over a transaction hash provided by the environment and a table creation counter
/// local to the transaction.
#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableHandle(pub ObjectID);

impl std::fmt::Display for TableHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<TableHandle> for ObjectID {
    fn from(table_handle: TableHandle) -> Self {
        table_handle.0
    }
}

impl FromStr for TableHandle {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TableHandle(ObjectID::from_str(s)?))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableTypeInfo {
    pub key_type: TypeTag,
}

impl TableTypeInfo {
    pub fn new(key_type: TypeTag) -> Self {
        Self { key_type }
    }
}

impl std::fmt::Display for TableTypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Table<{}>", self.key_type)
    }
}

/// A table change set.
#[derive(Default, Clone, Debug)]
pub struct TableChangeSet {
    pub new_tables: BTreeMap<TableHandle, TableTypeInfo>,
    pub removed_tables: BTreeSet<TableHandle>,
    pub changes: BTreeMap<TableHandle, TableChange>,
}

/// A change of a single table.
#[derive(Clone, Debug)]
pub struct TableChange {
    pub entries: BTreeMap<Vec<u8>, Op<State>>,
}

/// A table resolver which needs to be provided by the environment. This allows to lookup
/// data in remote storage, as well as retrieve cost of table operations.
pub trait TableResolver {
    fn resolve_table_entry(
        &self,
        handle: &TableHandle,
        key: &[u8],
    ) -> Result<Option<State>, anyhow::Error>;
}
