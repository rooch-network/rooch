// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
/// Public types for tables. The raw table implementation in moveos-stdlib natives.
/// We put them in moveos-types to avoid circular dependencies.
use crate::{object::ObjectID, state::State};
use move_core_types::{effects::Op, language_storage::TypeTag};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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
    pub new_tables: BTreeMap<ObjectID, TableTypeInfo>,
    pub removed_tables: BTreeSet<ObjectID>,
    pub changes: BTreeMap<ObjectID, TableChange>,
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
        handle: &ObjectID,
        key: &[u8],
    ) -> Result<Option<State>, anyhow::Error>;
}
