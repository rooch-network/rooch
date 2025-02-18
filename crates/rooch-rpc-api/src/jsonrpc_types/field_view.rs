// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{ObjectIDVecView, ObjectIDView, StrView};
use rooch_types::indexer::field::{FieldFilter, IndexerField};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct IndexerFieldView {
    pub id: ObjectIDView,
    pub field_key: String,
    pub name: String,
    pub value: StrView<u64>,
    // /// the field item created timestamp on chain
    // pub created_at: StrView<u64>,
    // /// the field item updated timestamp on chain
    // pub updated_at: StrView<u64>,
}

impl From<IndexerField> for IndexerFieldView {
    fn from(field: IndexerField) -> Self {
        IndexerFieldView {
            id: field.id.into(),
            field_key: field.field_key,
            name: field.name,
            value: field.value.into(),
            // created_at: field.created_at.into(),
            // updated_at: field.created_at.into(),
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
