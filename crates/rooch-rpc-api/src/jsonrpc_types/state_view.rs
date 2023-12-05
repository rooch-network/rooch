// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    AccountAddressView, AnnotatedMoveValueView, BytesView, RawObjectView, StrView, StructTagView,
    TypeTagView,
};
use move_core_types::effects::Op;
use move_core_types::language_storage::TypeTag;
use moveos_types::state::TableChangeSet;
use moveos_types::{
    moveos_std::object::ObjectID,
    state::{AnnotatedState, State, StateChangeSet, TableChange, TableTypeInfo},
};
use rooch_types::indexer::state::{
    GlobalStateFilter, IndexerGlobalState, IndexerStateChangeSet, IndexerTableChangeSet,
    IndexerTableState, StateSyncFilter, TableStateFilter,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

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
pub enum StateSyncFilterView {
    /// Query by table handle.
    TableHandle(ObjectID),
}

impl From<StateSyncFilterView> for StateSyncFilter {
    fn from(state_filter: StateSyncFilterView) -> Self {
        match state_filter {
            StateSyncFilterView::TableHandle(table_handle) => Self::TableHandle(table_handle),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct IndexerGlobalStateView {
    pub object_id: ObjectID,
    pub owner: AccountAddressView,
    pub flag: u8,
    pub value: RawObjectView,
    pub object_type: StructTagView,
    pub key_type: Option<TypeTagView>,
    pub size: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl IndexerGlobalStateView {
    pub fn try_new_from_global_state(
        state: IndexerGlobalState,
    ) -> Result<IndexerGlobalStateView, anyhow::Error> {
        let value: RawObjectView = serde_json::from_str(state.value.as_str())?;
        let key_type = if !state.key_type.is_empty() {
            Some(TypeTag::from_str(state.key_type.as_str())?.into())
        } else {
            None
        };
        let global_state_view = IndexerGlobalStateView {
            object_id: state.object_id,
            owner: state.owner.into(),
            flag: state.flag,
            value,
            object_type: state.object_type.into(),
            key_type,
            size: state.size,
            created_at: state.created_at,
            updated_at: state.updated_at,
        };
        Ok(global_state_view)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GlobalStateFilterView {
    /// Query by object type and owner.
    ObjectTypeWithOwner((StructTagView, AccountAddressView)),
    /// Query by object type.
    ObjectType(StructTagView),
    /// Query by owner.
    Owner(AccountAddressView),
    /// Query by object id.
    ObjectId(ObjectID),
}

impl From<GlobalStateFilterView> for GlobalStateFilter {
    fn from(state_filter: GlobalStateFilterView) -> Self {
        match state_filter {
            GlobalStateFilterView::ObjectTypeWithOwner((object_type, owner)) => {
                Self::ObjectTypeWithOwner((object_type.into(), owner.into()))
            }
            GlobalStateFilterView::ObjectType(object_type) => Self::ObjectType(object_type.into()),
            GlobalStateFilterView::Owner(owner) => Self::Owner(owner.into()),
            GlobalStateFilterView::ObjectId(object_id) => Self::ObjectId(object_id),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct IndexerTableStateView {
    pub id: String,
    pub table_handle: ObjectID,
    pub key_hex: String,
    pub value: AnnotatedMoveValueView,
    pub value_type: TypeTagView,
    pub created_at: u64,
    pub updated_at: u64,
}

impl IndexerTableStateView {
    pub fn try_new_from_table_state(
        state: IndexerTableState,
    ) -> Result<IndexerTableStateView, anyhow::Error> {
        let value: AnnotatedMoveValueView = serde_json::from_str(state.value.as_str())?;
        let state_view = IndexerTableStateView {
            id: state.id,
            table_handle: state.table_handle,
            key_hex: state.key_hex,
            value,
            value_type: state.value_type.into(),
            created_at: state.created_at,
            updated_at: state.updated_at,
        };
        Ok(state_view)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TableStateFilterView {
    /// Query by table handle.
    TableHandle(ObjectID),
}

impl From<TableStateFilterView> for TableStateFilter {
    fn from(state_filter: TableStateFilterView) -> Self {
        match state_filter {
            TableStateFilterView::TableHandle(table_handle) => Self::TableHandle(table_handle),
        }
    }
}
