// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{StrView, TypeTagView};
use move_core_types::effects::Op;
use moveos_types::table::{TableChange, TableChangeSet, TableHandle, TableTypeInfo, TableValue};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub type TableHandleView = StrView<TableHandle>;

impl_str_view_for!(TableHandle);

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
    pub new_tables: BTreeMap<TableHandleView, TableTypeInfoView>,
    pub removed_tables: BTreeSet<TableHandleView>,
    pub changes: BTreeMap<TableHandleView, TableChangeView>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableValueView {
    pub value_type: TypeTagView,
    pub value: StrView<Vec<u8>>,
}

impl From<TableValue> for TableValueView {
    fn from(table_value: TableValue) -> Self {
        Self {
            value_type: table_value.value_type.into(),
            value: table_value.value.into(),
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

impl From<Op<TableValue>> for OpView<TableValueView> {
    fn from(op: Op<TableValue>) -> Self {
        match op {
            Op::New(data) => Self::New(data.into()),
            Op::Modify(data) => Self::Modify(data.into()),
            Op::Delete => Self::Delete,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableChangeView {
    pub entries: BTreeMap<StrView<Vec<u8>>, OpView<TableValueView>>,
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
