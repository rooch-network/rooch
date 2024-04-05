// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    AccountAddressView, AnnotatedMoveStructView, AnnotatedMoveValueView, BytesView, StrView,
    StructTagView, TypeTagView,
};
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::Op;
use moveos_types::state::{
    AnnotatedKeyState, FieldChange, KeyState, NormalFieldChange, ObjectChange, TableChangeSet,
};
use moveos_types::state_resolver::StateKV;
use moveos_types::{
    moveos_std::object::ObjectID,
    state::{AnnotatedState, State, StateChangeSet, TableChange, TableTypeInfo},
};
use rooch_types::indexer::state::{
    FieldStateFilter, IndexerFieldState, IndexerObjectState, IndexerStateChangeSet,
    IndexerTableChangeSet, ObjectStateFilter, StateSyncFilter,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct DisplayFieldsView {
    pub fields: BTreeMap<String, String>,
}

impl DisplayFieldsView {
    pub fn new(fields: BTreeMap<String, String>) -> Self {
        Self { fields }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct StateView {
    pub value: BytesView,
    pub value_type: TypeTagView,
    pub decoded_value: Option<AnnotatedMoveValueView>,
    pub display_fields: Option<DisplayFieldsView>,
}

impl StateView {
    pub fn with_display_fields(mut self, display_fields: Option<DisplayFieldsView>) -> Self {
        self.display_fields = display_fields;
        self
    }
}

impl From<State> for StateView {
    fn from(state: State) -> Self {
        Self {
            value: StrView(state.value),
            value_type: state.value_type.into(),
            decoded_value: None,
            display_fields: None,
        }
    }
}

impl From<AnnotatedState> for StateView {
    fn from(state: AnnotatedState) -> Self {
        Self {
            value: StrView(state.state.value),
            value_type: state.state.value_type.into(),
            decoded_value: Some(state.decoded_value.into()),
            display_fields: None,
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

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SimpleKeyStateView {
    pub key: BytesView,
    pub key_type: TypeTagView,
}

impl From<KeyState> for SimpleKeyStateView {
    fn from(state: KeyState) -> Self {
        Self {
            key: StrView(state.key),
            key_type: state.key_type.into(),
        }
    }
}

impl From<KeyStateView> for SimpleKeyStateView {
    fn from(state: KeyStateView) -> Self {
        Self {
            key: state.key,
            key_type: state.key_type,
        }
    }
}

impl From<SimpleKeyStateView> for KeyState {
    fn from(state: SimpleKeyStateView) -> Self {
        Self {
            key: state.key.0,
            key_type: state.key_type.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Ord, Eq, PartialOrd, PartialEq)]
pub struct KeyStateView {
    pub key: BytesView,
    pub key_type: TypeTagView,
    pub decoded_key: Option<AnnotatedMoveValueView>,
}

impl From<KeyState> for KeyStateView {
    fn from(state: KeyState) -> Self {
        Self {
            key: StrView(state.key),
            key_type: state.key_type.into(),
            decoded_key: None,
        }
    }
}

impl From<AnnotatedKeyState> for KeyStateView {
    fn from(state: AnnotatedKeyState) -> Self {
        Self {
            key: StrView(state.state.key),
            key_type: state.state.key_type.into(),
            decoded_key: Some(state.decoded_key.into()),
        }
    }
}

impl From<KeyStateView> for KeyState {
    fn from(state: KeyStateView) -> Self {
        Self {
            key: state.key.0,
            key_type: state.key_type.into(),
        }
    }
}

impl std::fmt::Display for KeyStateView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key_state = KeyState::from(self.clone());
        write!(f, "{}", key_state)
    }
}

/// KeyStateView parse from str will ignored decoded_key
impl FromStr for KeyStateView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key_state = KeyState::from_str(s)?;
        Ok(KeyStateView::from(key_state))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct StateKVView {
    pub key_state: KeyStateView,
    pub state: StateView,
}

impl From<StateKV> for StateKVView {
    fn from(state: StateKV) -> Self {
        Self {
            key_state: state.0.into(),
            state: state.1.into(),
        }
    }
}

impl StateKVView {
    pub fn new(key_state: KeyStateView, state: StateView) -> Self {
        Self { key_state, state }
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

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateChangeSetView {
    pub global_size: u64,
    pub changes: BTreeMap<ObjectID, ObjectChangeView>,
}

impl From<StateChangeSet> for StateChangeSetView {
    fn from(state_change_set: StateChangeSet) -> Self {
        Self {
            global_size: state_change_set.global_size,
            changes: state_change_set
                .changes
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
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
// To support dynamic filed for json serialize and deserialize
pub struct DynamicFieldView {
    pub k: KeyStateView,
    pub v: OpView<StateView>,
}

impl DynamicFieldView {
    pub fn new(k: KeyStateView, v: OpView<StateView>) -> Self {
        Self { k, v }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FieldChangeView {
    Object {
        key: KeyStateView,
        #[serde(flatten)]
        change: ObjectChangeView,
    },
    Nomarl {
        key: KeyStateView,
        #[serde(flatten)]
        change: NormalFieldChangeView,
    },
}

impl From<(KeyState, FieldChange)> for FieldChangeView {
    fn from((key, field_change): (KeyState, FieldChange)) -> Self {
        match field_change {
            FieldChange::Object(object_change) => Self::Object {
                key: key.into(),
                change: object_change.into(),
            },
            FieldChange::Normal(normal_field_change) => Self::Nomarl {
                key: key.into(),
                change: normal_field_change.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct NormalFieldChangeView {
    pub op: OpView<StateView>,
}

impl From<NormalFieldChange> for NormalFieldChangeView {
    fn from(normal_field_change: NormalFieldChange) -> Self {
        Self {
            op: normal_field_change.op.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ObjectChangeView {
    pub op: Option<OpView<StateView>>,
    pub fields: Vec<FieldChangeView>,
}

impl From<ObjectChange> for ObjectChangeView {
    fn from(object_change: ObjectChange) -> Self {
        Self {
            op: object_change.op.map(|op| op.into()),
            fields: object_change
                .fields
                .into_iter()
                .map(|(k, v)| (k, v).into())
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

//TODO clean and remove TableChange
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TableChangeView {
    pub entries: Vec<DynamicFieldView>,
}

impl From<TableChange> for TableChangeView {
    fn from(table_change: TableChange) -> Self {
        Self {
            entries: table_change
                .entries
                .into_iter()
                .map(|(k, v)| DynamicFieldView::new(k.into(), v.into()))
                .collect::<Vec<_>>(),
        }
    }
}

impl From<TableChangeView> for TableChange {
    fn from(table_change: TableChangeView) -> Self {
        Self {
            entries: table_change
                .entries
                .into_iter()
                .map(|kv| (kv.k.into(), kv.v.into()))
                .collect(),
        }
    }
}
#[derive(Default, Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TableChangeSetView {
    pub new_tables: BTreeSet<ObjectID>,
    pub changes: BTreeMap<ObjectID, TableChangeView>,
    pub removed_tables: BTreeSet<ObjectID>,
}

impl From<TableChangeSet> for TableChangeSetView {
    fn from(table_change_set: TableChangeSet) -> Self {
        Self {
            new_tables: table_change_set.new_tables,
            removed_tables: table_change_set.removed_tables.into_iter().collect(),
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
            new_tables: table_change_set.new_tables,
            removed_tables: table_change_set
                .removed_tables
                .into_iter()
                .collect::<BTreeSet<_>>(),
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
    pub state_index: u64,
    pub table_handle: ObjectID,
    pub table_change_set: TableChangeSetView,
    pub created_at: u64,
}

impl From<IndexerTableChangeSet> for IndexerTableChangeSetView {
    fn from(table_change_set: IndexerTableChangeSet) -> Self {
        IndexerTableChangeSetView {
            tx_order: table_change_set.tx_order,
            state_index: table_change_set.state_index,
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
            StateSyncFilterView::TableHandle(table_handle) => Self::ObjectId(table_handle),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct IndexerObjectStateView {
    pub object_id: ObjectID,
    pub owner: AccountAddressView,
    pub flag: u8,
    pub value: AnnotatedMoveStructView,
    pub object_type: StructTagView,
    pub state_root: AccountAddressView,
    pub size: u64,
    pub tx_order: u64,
    pub state_index: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl IndexerObjectStateView {
    pub fn try_new_from_global_state(
        state: IndexerObjectState,
    ) -> Result<IndexerObjectStateView, anyhow::Error> {
        let value: AnnotatedMoveStructView = serde_json::from_str(state.value.as_str())?;
        let global_state_view = IndexerObjectStateView {
            object_id: state.object_id,
            owner: state.owner.into(),
            flag: state.flag,
            value,
            object_type: state.object_type.into(),
            state_root: state.state_root.into(),
            size: state.size,
            tx_order: state.tx_order,
            state_index: state.state_index,
            created_at: state.created_at,
            updated_at: state.updated_at,
        };
        Ok(global_state_view)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ObjectStateFilterView {
    /// Query by object value type and owner.
    ObjectTypeWithOwner {
        object_type: StructTagView,
        owner: AccountAddressView,
    },
    /// Query by object value type.
    ObjectType(StructTagView),
    /// Query by owner.
    Owner(AccountAddressView),
    /// Query by object id.
    ObjectId(ObjectID),
    /// Query by multi chain address
    MultiChainAddress { multichain_id: u64, address: String },
}

impl ObjectStateFilterView {
    pub fn into_object_state_filter(
        state_filter: ObjectStateFilterView,
        resolve_address: AccountAddress,
    ) -> ObjectStateFilter {
        match state_filter {
            ObjectStateFilterView::ObjectTypeWithOwner { object_type, owner } => {
                ObjectStateFilter::ObjectTypeWithOwner {
                    object_type: object_type.into(),
                    owner: owner.into(),
                }
            }
            ObjectStateFilterView::ObjectType(object_type) => {
                ObjectStateFilter::ObjectType(object_type.into())
            }
            ObjectStateFilterView::Owner(owner) => ObjectStateFilter::Owner(owner.into()),
            ObjectStateFilterView::ObjectId(object_id) => ObjectStateFilter::ObjectId(object_id),
            ObjectStateFilterView::MultiChainAddress {
                multichain_id: _,
                address: _,
            } => ObjectStateFilter::Owner(resolve_address),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct IndexerFieldStateView {
    pub object_id: ObjectID,
    pub key_hex: String,
    pub key: AnnotatedMoveValueView,
    pub value: AnnotatedMoveValueView,
    pub key_type: TypeTagView,
    pub value_type: TypeTagView,
    pub tx_order: u64,
    pub state_index: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl IndexerFieldStateView {
    pub fn try_new_from_table_state(
        state: IndexerFieldState,
    ) -> Result<IndexerFieldStateView, anyhow::Error> {
        let key: AnnotatedMoveValueView = serde_json::from_str(state.key_str.as_str())?;
        let value: AnnotatedMoveValueView = serde_json::from_str(state.value.as_str())?;
        let state_view = IndexerFieldStateView {
            object_id: state.object_id,
            key_hex: state.key_hex,
            key,
            value,
            key_type: state.key_type.into(),
            value_type: state.value_type.into(),
            tx_order: state.tx_order,
            state_index: state.state_index,
            created_at: state.created_at,
            updated_at: state.updated_at,
        };
        Ok(state_view)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FieldStateFilterView {
    /// Query by object id.
    ObjectId(ObjectID),
}

impl From<FieldStateFilterView> for FieldStateFilter {
    fn from(state_filter: FieldStateFilterView) -> Self {
        match state_filter {
            FieldStateFilterView::ObjectId(object_id) => Self::ObjectId(object_id),
        }
    }
}
