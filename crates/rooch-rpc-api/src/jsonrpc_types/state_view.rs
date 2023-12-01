// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{AnnotatedMoveValueView, BytesView, StrView, TypeTagView};
use move_core_types::effects::Op;
use moveos_types::state::TableChangeSet;
use moveos_types::{
    moveos_std::object::ObjectID,
    state::{AnnotatedState, State, StateChangeSet, TableChange, TableTypeInfo},
};
use rooch_types::indexer::state::{IndexerStateChangeSet, IndexerTableChangeSet, StateFilter};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct StateView {
    pub value: BytesView,
    pub value_type: TypeTagView,
    pub decoded_value: Option<AnnotatedMoveValueView>,
}

impl From<State> for StateView {
    fn from(state: State) -> Self {
        Self {
            value: StrView(state.value),
            value_type: state.value_type.into(),
            decoded_value: None,
        }
    }
}

impl From<AnnotatedState> for StateView {
    fn from(state: AnnotatedState) -> Self {
        Self {
            value: StrView(state.state.value),
            value_type: state.state.value_type.into(),
            decoded_value: Some(state.decoded_value.into()),
        }
    }
}

impl From<StateView> for State {
    fn from(state: StateView) -> Self {
        Self {
            value: state.value.0,
            value_type: state.value_type.into(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
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

impl From<TableTypeInfoView> for TableTypeInfo {
    fn from(table_type_info: TableTypeInfoView) -> Self {
        Self {
            key_type: table_type_info.key_type.into(),
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct StateChangeSetView {
    pub new_tables: BTreeMap<ObjectID, TableTypeInfoView>,
    pub removed_tables: BTreeSet<ObjectID>,
    pub changes: BTreeMap<ObjectID, TableChangeView>,
}

impl From<StateChangeSet> for StateChangeSetView {
    fn from(table_change_set: StateChangeSet) -> Self {
        Self {
            new_tables: table_change_set
                .new_tables
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            removed_tables: table_change_set.removed_tables,
            changes: table_change_set
                .changes
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

impl From<OpView<StateView>> for Op<State> {
    fn from(op: OpView<StateView>) -> Self {
        match op {
            OpView::New(data) => Self::New(data.into()),
            OpView::Modify(data) => Self::Modify(data.into()),
            OpView::Delete => Self::Delete,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TableChangeView {
    pub entries: BTreeMap<BytesView, OpView<StateView>>,
    pub size_increment: i64,
}

impl From<TableChange> for TableChangeView {
    fn from(table_change: TableChange) -> Self {
        Self {
            entries: table_change
                .entries
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
            size_increment: table_change.size_increment,
        }
    }
}

impl From<TableChangeView> for TableChange {
    fn from(table_change: TableChangeView) -> Self {
        Self {
            entries: table_change
                .entries
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
            size_increment: table_change.size_increment,
        }
    }
}

impl From<StateChangeSetView> for StateChangeSet {
    fn from(table_change_set: StateChangeSetView) -> Self {
        Self {
            new_tables: table_change_set
                .new_tables
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            removed_tables: table_change_set.removed_tables,
            changes: table_change_set
                .changes
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct IndexerStateChangeSetView {
    pub tx_order: u64,
    pub state_change_set: StateChangeSetView,
    pub created_at: u64,
}

impl From<IndexerStateChangeSet> for IndexerStateChangeSetView {
    fn from(state_change_set: IndexerStateChangeSet) -> Self {
        IndexerStateChangeSetView {
            tx_order: state_change_set.tx_order,
            state_change_set: state_change_set.state_change_set.into(),
            created_at: state_change_set.created_at,
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, JsonSchema)]
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
                .map(|(k, v)| (k, v.into()))
                .collect(),
            removed_tables: table_change_set.removed_tables,
            changes: table_change_set
                .changes
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

impl From<TableChangeSetView> for TableChangeSet {
    fn from(table_change_set: TableChangeSetView) -> Self {
        Self {
            new_tables: table_change_set
                .new_tables
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            removed_tables: table_change_set.removed_tables,
            changes: table_change_set
                .changes
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct IndexerTableChangeSetView {
    pub tx_order: u64,
    pub table_handle_index: u64,
    pub table_handle: ObjectID,
    pub table_change_set: TableChangeSetView,
    pub created_at: u64,
}

impl From<IndexerTableChangeSet> for IndexerTableChangeSetView {
    fn from(table_change_set: IndexerTableChangeSet) -> Self {
        IndexerTableChangeSetView {
            tx_order: table_change_set.tx_order,
            table_handle_index: table_change_set.table_handle_index,
            table_handle: table_change_set.table_handle,
            table_change_set: table_change_set.table_change_set.into(),
            created_at: table_change_set.created_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StateFilterView {
    /// Query by table handle.
    TableHandle(ObjectID),
}

impl From<StateFilterView> for StateFilter {
    fn from(state_filter: StateFilterView) -> Self {
        match state_filter {
            StateFilterView::TableHandle(table_handle) => Self::TableHandle(table_handle),
        }
    }
}
