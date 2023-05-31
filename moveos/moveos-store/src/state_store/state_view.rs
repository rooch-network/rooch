// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, language_storage::StructTag, resolver::MoveResolver,
};
use move_resource_viewer::MoveValueAnnotator;
use moveos_types::{
    access_path::AccessPath,
    object::{AnnotatedObject, ObjectID},
    state::{AnnotatedState, State},
};

/// StateReader provide an unify State API with AccessPath
pub trait StateReader {
    /// Get states by AccessPath
    fn get_states(&self, path: &AccessPath) -> Result<Vec<Option<State>>>;
}

pub trait AnnotatedStateReader: StateReader + MoveResolver {
    fn get_annotated_states(&self, path: &AccessPath) -> Result<Vec<Option<AnnotatedState>>> {
        let annotator = MoveValueAnnotator::new(self);
        self.get_states(path)?
            .into_iter()
            .map(|state| {
                state
                    .map(|state| state.into_annotated_state(&annotator))
                    .transpose()
            })
            .collect()
    }

    fn get_annotated_object(&self, object_id: ObjectID) -> Result<Option<AnnotatedObject>> {
        let annotator = MoveValueAnnotator::new(self);
        self.get_states(&AccessPath::object(object_id))?
            .pop()
            .and_then(|state| state)
            .map(|state| {
                state
                    .into_annotated_state(&annotator)?
                    .into_annotated_object()
            })
            .transpose()
    }

    fn get_annotated_resource(
        &self,
        account: AccountAddress,
        resource_type: StructTag,
    ) -> Result<Option<AnnotatedState>> {
        let annotator = MoveValueAnnotator::new(self);
        self.get_states(&AccessPath::resource(account, resource_type))?
            .pop()
            .and_then(|state| state)
            .map(|state| state.into_annotated_state(&annotator))
            .transpose()
    }
}

impl<T> AnnotatedStateReader for T where T: StateReader + MoveResolver {}
