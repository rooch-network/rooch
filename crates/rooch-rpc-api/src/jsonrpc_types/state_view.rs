// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{AnnotatedMoveValueView, StrView, TypeTagView};
use move_core_types::effects::Op;
use moveos_types::{
    object::ObjectID,
    state::{AnnotatedState, State, StateChangeSet, TableChange, TableTypeInfo},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct StateView {
    pub value: StrView<Vec<u8>>,
    pub value_type: TypeTagView,
}

impl From<State> for StateView {
    fn from(state: State) -> Self {
        Self {
            value: StrView(state.value),
            value_type: state.value_type.into(),
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
pub struct AnnotatedStateView {
    pub state: StateView,
    pub move_value: AnnotatedMoveValueView,
}

impl From<AnnotatedState> for AnnotatedStateView {
    fn from(state: AnnotatedState) -> Self {
        Self {
            state: state.state.into(),
            move_value: state.move_value.into(),
        }
    }
}

//TODO Is it need to convert the AnnotatedStateView back to AnnotatedState?
//If not, please remove this code. Otherwise, it needs to be fixed. include TryFrom<AnnotatedMoveValueView> for AnnotatedMoveValue
// impl TryFrom<AnnotatedStateView> for AnnotatedState {
//     type Error = anyhow::Error;

//     fn try_from(value: AnnotatedStateView) -> Result<Self, Self::Error> {
//         Ok(Self {
//             state: value.state.into(),
//             move_value: value.move_value.try_into()?,
//         })
//     }
// }

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

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
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
