// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    AnnotatedMoveStructView, AnnotatedMoveValueView, BytesView, H256View, HumanReadableDisplay,
    ObjectIDVecView, RoochAddressView, RoochOrBitcoinAddressView, StrView, StructTagView,
    TypeTagView,
};
use anyhow::Result;

use bcs;
use move_core_types::effects::Op;
use moveos_types::state::{
    AnnotatedKeyState, FieldChange, KeyState, NormalFieldChange, ObjectChange,
};
use moveos_types::state_resolver::StateKV;
use moveos_types::{
    moveos_std::object::{human_readable_flag, AnnotatedObject, ObjectID, RawObject},
    state::{AnnotatedState, State, StateChangeSet, TableTypeInfo},
};
use rooch_types::indexer::state::{
    IndexerObjectState, IndexerStateChangeSet, ObjectStateFilter, StateSyncFilter,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Ord, Eq, PartialOrd, PartialEq)]
pub struct KeyStateHexView(pub BytesView);

impl From<KeyState> for KeyStateHexView {
    fn from(state: KeyState) -> Self {
        let bytes = bcs::to_bytes(&state).expect("bcs serialization should succeed");
        Self(bytes.into())
    }
}

impl From<AnnotatedKeyState> for KeyStateHexView {
    fn from(state: AnnotatedKeyState) -> Self {
        state.state.into()
    }
}

impl From<KeyStateHexView> for KeyState {
    fn from(state: KeyStateHexView) -> Self {
        let bytes = state.0 .0;
        bcs::from_bytes(&bytes).expect("bcs deserialization should succeed")
    }
}

impl std::fmt::Display for KeyStateHexView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0 .0))
    }
}

// TODO: Do we need to keep KeyStateView for showing key state in detail?

// #[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Ord, Eq, PartialOrd, PartialEq)]
// pub struct KeyStateView {
//     pub key: BytesView,
//     pub key_type: TypeTagView,
//     pub decoded_key: Option<AnnotatedMoveValueView>,
// }

// impl From<KeyState> for KeyStateView {
//     fn from(state: KeyState) -> Self {
//         Self {
//             key: StrView(state.key),
//             key_type: state.key_type.into(),
//             decoded_key: None,
//         }
//     }
// }

// impl From<AnnotatedKeyState> for KeyStateView {
//     fn from(state: AnnotatedKeyState) -> Self {
//         Self {
//             key: StrView(state.state.key),
//             key_type: state.state.key_type.into(),
//             decoded_key: Some(state.decoded_key.into()),
//         }
//     }
// }

// impl From<KeyStateView> for KeyState {
//     fn from(state: KeyStateView) -> Self {
//         Self {
//             key: state.key.0,
//             key_type: state.key_type.into(),
//         }
//     }
// }

// impl std::fmt::Display for KeyStateView {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let key_state = KeyState::from(self.clone());
//         write!(f, "{}", key_state)
//     }
// }

// /// KeyStateView parse from str will ignored decoded_key
// impl FromStr for KeyStateView {
//     type Err = anyhow::Error;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let key_state = KeyState::from_str(s)?;
//         Ok(KeyStateView::from(key_state))
//     }
// }

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct StateKVView {
    pub key_hex: KeyStateHexView,
    pub state: StateView,
}

impl From<StateKV> for StateKVView {
    fn from(state: StateKV) -> Self {
        Self {
            key_hex: state.0.into(),
            state: state.1.into(),
        }
    }
}

impl StateKVView {
    pub fn new(key_hex: KeyStateHexView, state: StateView) -> Self {
        Self { key_hex, state }
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FieldChangeView {
    Object {
        key: KeyStateHexView,
        key_state: String,
        #[serde(flatten)]
        change: ObjectChangeView,
    },
    Normal {
        key: KeyStateHexView,
        key_state: String,
        #[serde(flatten)]
        change: NormalFieldChangeView,
    },
}

impl From<(KeyState, FieldChange)> for FieldChangeView {
    fn from((key, field_change): (KeyState, FieldChange)) -> Self {
        match field_change {
            FieldChange::Object(object_change) => Self::Object {
                key: key.clone().into(),
                key_state: key.to_string(),
                change: object_change.into(),
            },
            FieldChange::Normal(normal_field_change) => Self::Normal {
                key: key.clone().into(),
                key_state: key.to_string(),
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

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StateSyncFilterView {
    /// Query by table handle.
    TableHandle(ObjectID),
}

impl From<StateSyncFilterView> for StateSyncFilter {
    fn from(state_filter: StateSyncFilterView) -> Self {
        match state_filter {
            StateSyncFilterView::TableHandle(object_id) => Self::ObjectId(object_id),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct IndexerObjectStateView {
    pub object_id: ObjectID,
    pub owner: RoochAddressView,
    pub owner_bitcoin_address: Option<String>,
    pub flag: u8,
    /// bcs bytes of the Object.
    pub value: BytesView,
    pub decoded_value: Option<AnnotatedMoveStructView>,
    pub object_type: StructTagView,
    pub state_root: H256View,
    pub size: u64,
    pub tx_order: u64,
    pub state_index: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub display_fields: Option<DisplayFieldsView>,
}

impl IndexerObjectStateView {
    pub fn new_from_object_state(
        state: IndexerObjectState,
        value: Vec<u8>,
        owner_bitcoin_address: Option<String>,
        decoded_value: Option<AnnotatedMoveStructView>,
        display_fields: Option<DisplayFieldsView>,
    ) -> IndexerObjectStateView {
        IndexerObjectStateView {
            object_id: state.object_id,
            owner: state.owner.into(),
            owner_bitcoin_address,
            flag: state.flag,
            value: value.into(),
            decoded_value,
            object_type: state.object_type.into(),
            state_root: state.state_root.into(),
            size: state.size,
            tx_order: state.tx_order,
            state_index: state.state_index,
            created_at: state.created_at,
            updated_at: state.updated_at,
            display_fields,
        }
    }
}

impl HumanReadableDisplay for IndexerObjectStateView {
    fn to_human_readable_string(&self, verbose: bool) -> String {
        let _ = verbose; // TODO: implement verbose string

        format!(
            r#"{}
  objectId       | {}
  type           | {}
  owner          | {}
  owner(bitcoin) | {:?}
  status         | {}
  tx_order       | {}
  state_index    | {}
{}"#,
            "-".repeat(100),
            self.object_id,
            self.object_type,
            self.owner,
            self.owner_bitcoin_address,
            human_readable_flag(self.flag),
            self.tx_order,
            self.state_index,
            "-".repeat(100),
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ObjectStateFilterView {
    /// Query by object value type and owner.
    ObjectTypeWithOwner {
        object_type: StructTagView,
        owner: RoochOrBitcoinAddressView,
    },
    /// Query by object value type.
    ObjectType(StructTagView),
    /// Query by owner.
    Owner(RoochOrBitcoinAddressView),
    /// Query by object ids.
    ObjectId(ObjectIDVecView),
}

impl ObjectStateFilterView {
    pub fn try_into_object_state_filter(
        state_filter: ObjectStateFilterView,
    ) -> Result<ObjectStateFilter> {
        Ok(match state_filter {
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
            ObjectStateFilterView::ObjectId(object_id_vec_view) => {
                ObjectStateFilter::ObjectId(object_id_vec_view.into())
            }
        })
    }
}

/// Object state view. Used as return type of `getObjectStates`.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ObjectStateView {
    pub id: ObjectID,
    pub owner: RoochAddressView,
    pub owner_bitcoin_address: Option<String>,
    pub flag: u8,
    pub object_type: StructTagView,
    pub state_root: H256View,
    pub size: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub value: BytesView,
    pub decoded_value: Option<AnnotatedMoveStructView>,
    pub display_fields: Option<DisplayFieldsView>,
}

impl ObjectStateView {
    pub fn new(value: Vec<u8>, object: AnnotatedObject, decode: bool) -> Self {
        ObjectStateView {
            id: object.id,
            owner: object.owner.into(),
            owner_bitcoin_address: None,
            flag: object.flag,
            object_type: object.value.type_.clone().into(),
            state_root: object.state_root.into(),
            size: object.size,
            created_at: object.created_at,
            updated_at: object.updated_at,
            value: value.into(),
            decoded_value: if decode {
                Some(AnnotatedMoveStructView::from(object.value))
            } else {
                None
            },
            display_fields: None,
        }
    }

    pub fn new_from_raw_object(object: RawObject) -> Self {
        ObjectStateView {
            id: object.id,
            owner: object.owner.into(),
            owner_bitcoin_address: None,
            flag: object.flag,
            object_type: object.value.struct_tag.into(),
            state_root: object.state_root.into(),
            size: object.size,
            created_at: object.created_at,
            updated_at: object.updated_at,
            value: object.value.value.into(),
            decoded_value: None,
            display_fields: None,
        }
    }

    pub fn with_display_fields(mut self, display_fields: Option<DisplayFieldsView>) -> Self {
        self.display_fields = display_fields;
        self
    }

    pub fn with_owner_bitcoin_address(mut self, owner_bitcoin_address: Option<String>) -> Self {
        self.owner_bitcoin_address = owner_bitcoin_address;
        self
    }
}

impl HumanReadableDisplay for ObjectStateView {
    fn to_human_readable_string(&self, verbose: bool) -> String {
        let _ = verbose; // TODO: implement verbose string

        format!(
            r#"{}
  objectId       | {}
  type           | {}
  owner          | {}
  owner(bitcoin) | {:?}
  status         | {}
{}"#,
            "-".repeat(100),
            self.id,
            self.object_type,
            self.owner,
            self.owner_bitcoin_address,
            human_readable_flag(self.flag),
            "-".repeat(100),
        )
    }
}

impl<T> HumanReadableDisplay for Vec<T>
where
    T: HumanReadableDisplay,
{
    fn to_human_readable_string(&self, verbose: bool) -> String {
        let _ = verbose;
        self.iter()
            .map(|s| s.to_human_readable_string(verbose))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
