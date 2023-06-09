// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{StateView, StrView, TypeTagView};
use move_core_types::effects::Op;
use moveos_types::{
    object::ObjectID,
    state::State,
    table::{TableChange, TableChangeSet, TableTypeInfo},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableTypeInfoView {
    pub key_type: TypeTagView,
}

impl From<TableTypeInfo> for TableTypeInfoView {
    fn from(table_type_info: TableTypeInfo) -> Self {
        Self {
            key_type: table_type_info.key_type.into(),
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct TableChangeSetView {
    pub new_tables: BTreeMap<ObjectID, TableTypeInfoView>,
    pub removed_tables: BTreeSet<ObjectID>,
    pub changes: BTreeMap<ObjectID, TableChangeView>,
}

impl From<TableChangeSet> for TableChangeSetView {
    fn from(table_change_set: TableChangeSet) -> Self {
        Self {
            new_tables: table_change_set
                .new_tables
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
            removed_tables: table_change_set
                .removed_tables
                .into_iter()
                .map(|k| k.into())
                .collect(),
            changes: table_change_set
                .changes
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OpView<T> {
    New(T),
    Modify(T),
    Delete,
}

impl From<Op<State>> for OpView<StateView> {
    fn from(op: Op<State>) -> Self {
        match op {
            Op::New(data) => Self::New(data.into()),
            Op::Modify(data) => Self::Modify(data.into()),
            Op::Delete => Self::Delete,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableChangeView {
    pub entries: BTreeMap<StrView<Vec<u8>>, OpView<StateView>>,
}

impl From<TableChange> for TableChangeView {
    fn from(table_change: TableChange) -> Self {
        Self {
            entries: table_change
                .entries
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }
}
