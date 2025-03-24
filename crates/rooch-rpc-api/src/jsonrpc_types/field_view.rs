// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    AnnotatedMoveStructView, FieldKeyView, ObjectIDVecView, ObjectStateView, StrView,
};
use moveos_types::state::{AnnotatedState, ObjectState};
use rooch_types::indexer::field::{FieldFilter, IndexerField};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct IndexerFieldView {
    pub field_key: FieldKeyView,
    pub state: ObjectStateView,
    pub sort_key: String,
    // /// the field item created timestamp on chain
    // pub created_at: StrView<u64>,
    // /// the field item updated timestamp on chain
    // pub updated_at: StrView<u64>,
    pub decoded_value: Option<serde_json::Value>,
}

impl IndexerFieldView {
    pub fn new_from_state(field: IndexerField, state: ObjectState) -> Self {
        IndexerFieldView {
            field_key: field.field_key.into(),
            state: state.into(),
            sort_key: StrView(field.sort_key).to_string(),
            decoded_value: None,
        }
    }

    pub fn new_from_annotated_state(
        field: IndexerField,
        annotated_state: AnnotatedState,
    ) -> IndexerFieldView {
        let (metadata, value, decoded_value) = annotated_state.into_inner();
        let state = ObjectState::new(metadata, value);
        IndexerFieldView {
            field_key: field.field_key.into(),
            state: state.into(),
            sort_key: StrView(field.sort_key).to_string(),
            decoded_value: Some(AnnotatedMoveStructView::from(decoded_value).into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FieldFilterView {
    /// Query by object ids.
    ObjectId(ObjectIDVecView),
}

impl From<FieldFilterView> for FieldFilter {
    fn from(field_filter: FieldFilterView) -> Self {
        match field_filter {
            FieldFilterView::ObjectId(ids) => Self::ObjectId(ids.into()),
        }
    }
}
