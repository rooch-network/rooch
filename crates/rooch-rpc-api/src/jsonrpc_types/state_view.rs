// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    AnnotatedMoveStructView, BytesView, H256View, HumanReadableDisplay, ObjectIDVecView,
    RoochAddressView, RoochOrBitcoinAddressView, StrView, StructTagView, TypeTagView,
};
use anyhow::Result;
use move_core_types::effects::Op;
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::state::{FieldKey, ObjectChange};
use moveos_types::state_resolver::StateKV;
use moveos_types::{
    moveos_std::object::{human_readable_flag, ObjectID},
    state::{AnnotatedState, ObjectState, StateChangeSet},
};
use rooch_types::indexer::state::{IndexerStateID, ObjectStateFilter};
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

pub type FieldKeyView = StrView<FieldKey>;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct StateKVView {
    pub field_key: FieldKeyView,
    pub state: ObjectStateView,
}

impl From<StateKV> for StateKVView {
    fn from(state: StateKV) -> Self {
        Self {
            field_key: state.0.into(),
            state: state.1.into(),
        }
    }
}

impl StateKVView {
    pub fn new(field_key: FieldKeyView, state: ObjectStateView) -> Self {
        Self { field_key, state }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ObjectMetaView {
    pub id: ObjectID,
    pub owner: RoochAddressView,
    pub owner_bitcoin_address: Option<String>,
    pub flag: u8,
    pub state_root: Option<H256View>,
    pub size: StrView<u64>,
    pub created_at: StrView<u64>,
    pub updated_at: StrView<u64>,
    pub object_type: TypeTagView,
}

impl ObjectMetaView {
    pub fn with_owner_bitcoin_address(mut self, owner_bitcoin_address: Option<String>) -> Self {
        self.owner_bitcoin_address = owner_bitcoin_address;
        self
    }
}

impl From<ObjectMeta> for ObjectMetaView {
    fn from(meta: ObjectMeta) -> Self {
        Self {
            id: meta.id,
            owner: meta.owner.into(),
            owner_bitcoin_address: None,
            flag: meta.flag,
            state_root: meta.state_root.map(Into::into),
            size: meta.size.into(),
            created_at: meta.created_at.into(),
            updated_at: meta.updated_at.into(),
            object_type: meta.object_type.into(),
        }
    }
}

impl From<ObjectMetaView> for ObjectMeta {
    fn from(meta: ObjectMetaView) -> Self {
        Self {
            id: meta.id,
            owner: meta.owner.into(),
            flag: meta.flag,
            state_root: meta.state_root.map(Into::into),
            size: meta.size.0,
            created_at: meta.created_at.0,
            updated_at: meta.updated_at.0,
            object_type: meta.object_type.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateChangeSetView {
    pub root_metadata: ObjectMetaView,
    pub changes: Vec<ObjectChangeView>,
}

impl From<StateChangeSet> for StateChangeSetView {
    fn from(state_change_set: StateChangeSet) -> Self {
        Self {
            root_metadata: state_change_set.root_metadata.into(),
            changes: state_change_set
                .changes
                .into_values()
                .map(|v| v.into())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OpView {
    New(BytesView),
    Modify(BytesView),
    Delete,
}

impl From<OpView> for Op<Vec<u8>> {
    fn from(op: OpView) -> Self {
        match op {
            OpView::New(data) => Self::New(data.0),
            OpView::Modify(data) => Self::Modify(data.0),
            OpView::Delete => Self::Delete,
        }
    }
}

impl From<Op<Vec<u8>>> for OpView {
    fn from(op: Op<Vec<u8>>) -> Self {
        match op {
            Op::New(data) => Self::New(data.into()),
            Op::Modify(data) => Self::Modify(data.into()),
            Op::Delete => Self::Delete,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ObjectChangeView {
    pub metadata: ObjectMetaView,
    pub value: Option<OpView>,
    pub fields: Vec<ObjectChangeView>,
}

impl From<ObjectChange> for ObjectChangeView {
    fn from(object_change: ObjectChange) -> Self {
        Self {
            metadata: object_change.metadata.into(),
            value: object_change.value.map(|op| op.into()),
            fields: object_change
                .fields
                .into_values()
                .map(|v| v.into())
                .collect(),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct IndexerStateIDView {
    pub tx_order: StrView<u64>,
    pub state_index: StrView<u64>,
}

impl From<IndexerStateID> for IndexerStateIDView {
    fn from(id: IndexerStateID) -> Self {
        Self {
            tx_order: id.tx_order.into(),
            state_index: id.state_index.into(),
        }
    }
}

impl From<IndexerStateIDView> for IndexerStateID {
    fn from(id: IndexerStateIDView) -> Self {
        Self {
            tx_order: id.tx_order.0,
            state_index: id.state_index.0,
        }
    }
}

impl std::fmt::Display for IndexerStateIDView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IndexerStateID[tx order: {}, state index: {}]",
            self.tx_order, self.state_index,
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct IndexerObjectStateView {
    #[serde(flatten)]
    pub metadata: ObjectMetaView,
    /// bcs bytes of the Object.
    pub value: BytesView,
    pub decoded_value: Option<AnnotatedMoveStructView>,
    #[serde(flatten)]
    pub indexer_id: IndexerStateIDView,
    pub display_fields: Option<DisplayFieldsView>,
}

impl IndexerObjectStateView {
    pub fn new_from_object_state(
        state: ObjectState,
        indexer_id: IndexerStateID,
    ) -> IndexerObjectStateView {
        let (metadata, value) = state.into_inner();
        IndexerObjectStateView {
            metadata: metadata.into(),
            value: value.into(),
            decoded_value: None,
            indexer_id: indexer_id.into(),
            display_fields: None,
        }
    }

    pub fn new_from_annotated_state(
        state: AnnotatedState,
        indexer_id: IndexerStateID,
    ) -> IndexerObjectStateView {
        let (metadata, value, decoded_value) = state.into_inner();
        IndexerObjectStateView {
            metadata: metadata.into(),
            value: value.into(),
            decoded_value: Some(AnnotatedMoveStructView::from(decoded_value)),
            indexer_id: indexer_id.into(),
            display_fields: None,
        }
    }

    pub fn with_owner_bitcoin_address(mut self, owner_bitcoin_address: Option<String>) -> Self {
        self.metadata.owner_bitcoin_address = owner_bitcoin_address;
        self
    }

    pub fn with_display_fields(mut self, display_fields: Option<DisplayFieldsView>) -> Self {
        self.display_fields = display_fields;
        self
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
            self.metadata.id,
            self.metadata.object_type,
            self.metadata.owner,
            self.metadata.owner_bitcoin_address,
            human_readable_flag(self.metadata.flag),
            self.indexer_id.tx_order,
            self.indexer_id.state_index,
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
    #[serde(flatten)]
    pub metadata: ObjectMetaView,
    pub value: BytesView,
    pub decoded_value: Option<AnnotatedMoveStructView>,
    pub display_fields: Option<DisplayFieldsView>,
}

impl ObjectStateView {
    pub fn new(object: AnnotatedState, decode: bool) -> Self {
        let (metadata, value, decoded_value) = object.into_inner();

        ObjectStateView {
            metadata: metadata.into(),
            value: value.into(),
            decoded_value: if decode {
                Some(AnnotatedMoveStructView::from(decoded_value))
            } else {
                None
            },
            display_fields: None,
        }
    }

    pub fn new_from_object_state(object: ObjectState) -> Self {
        let (metadata, value) = object.into_inner();
        ObjectStateView {
            metadata: metadata.into(),
            value: value.into(),
            decoded_value: None,
            display_fields: None,
        }
    }

    pub fn with_display_fields(mut self, display_fields: Option<DisplayFieldsView>) -> Self {
        self.display_fields = display_fields;
        self
    }

    pub fn with_owner_bitcoin_address(mut self, owner_bitcoin_address: Option<String>) -> Self {
        self.metadata.owner_bitcoin_address = owner_bitcoin_address;
        self
    }
}

impl From<ObjectState> for ObjectStateView {
    fn from(state: ObjectState) -> Self {
        ObjectStateView::new_from_object_state(state)
    }
}

impl From<ObjectStateView> for ObjectState {
    fn from(state: ObjectStateView) -> Self {
        let metadata = state.metadata.into();
        ObjectState::new(metadata, state.value.0)
    }
}

impl From<AnnotatedState> for ObjectStateView {
    fn from(state: AnnotatedState) -> Self {
        ObjectStateView::new(state, true)
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
            self.metadata.id,
            self.metadata.object_type,
            self.metadata.owner,
            self.metadata.owner_bitcoin_address,
            human_readable_flag(self.metadata.flag),
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
