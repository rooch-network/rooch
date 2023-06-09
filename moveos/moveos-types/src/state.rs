// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    object::{self, AnnotatedObject, Object, ObjectID, RawObject},
};
use anyhow::{bail, ensure, Result};
use move_core_types::{
    effects::Op,
    language_storage::{StructTag, TypeTag},
    move_resource::MoveStructType,
    resolver::MoveResolver,
    value::MoveStructLayout,
};
use move_resource_viewer::{AnnotatedMoveValue, MoveValueAnnotator};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// `State` is represent state in MoveOS statedb, it can be a Move module or a Move Object or a Move resource or a Table value
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct State {
    /// the bytes of state
    pub value: Vec<u8>,
    /// the type of state
    pub value_type: TypeTag,
}

/// Move State is a trait that is used to represent the state of a Move Resource in Rust
/// It is like the `MoveResource` in move_core_types
pub trait MoveState: MoveStructType + DeserializeOwned + Serialize {
    fn move_layout() -> MoveStructLayout;
    fn type_match(type_tag: &StructTag) -> bool {
        type_tag == &Self::struct_tag()
    }
}

impl State {
    pub fn new(value: Vec<u8>, value_type: TypeTag) -> Self {
        Self { value, value_type }
    }

    pub fn value_type(&self) -> &TypeTag {
        &self.value_type
    }

    pub fn match_type(&self, type_tag: &TypeTag) -> bool {
        &self.value_type == type_tag
    }

    pub fn match_struct_type(&self, type_tag: &StructTag) -> bool {
        match &self.value_type {
            TypeTag::Struct(struct_tag) => struct_tag.as_ref() == type_tag,
            _ => false,
        }
    }

    pub fn is_object(&self) -> bool {
        self.get_object_struct_tag().is_some()
    }

    /// If the state is a Object, return the T's struct_tag of Object<T>
    /// Otherwise, return None
    pub fn get_object_struct_tag(&self) -> Option<StructTag> {
        let val_type = self.value_type();
        match val_type {
            TypeTag::Struct(struct_tag) => {
                if struct_tag.address == *MOVEOS_STD_ADDRESS
                    && struct_tag.module.as_ident_str() == object::OBJECT_MODULE_NAME
                    && struct_tag.name.as_ident_str() == object::OBJECT_STRUCT_NAME
                {
                    let object_type_param = struct_tag
                        .type_params
                        .get(0)
                        .expect("The Object<T> should have a type param");
                    match object_type_param {
                        TypeTag::Struct(struct_tag) => Some(struct_tag.as_ref().clone()),
                        _ => {
                            unreachable!("The Object<T> should have a struct type param")
                        }
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn as_object<T>(&self) -> Result<Object<T>>
    where
        T: MoveState,
    {
        self.as_move_state::<Object<T>>()
    }

    pub fn as_raw_object(&self) -> Result<RawObject> {
        let object_struct_tag = self.get_object_struct_tag().ok_or_else(|| {
            anyhow::anyhow!("Expect type Object but the state type:{}", self.value_type)
        })?;
        RawObject::from_bytes(&self.value, object_struct_tag)
    }

    pub fn as_move_state<T>(&self) -> Result<T>
    where
        T: MoveState,
    {
        let val_type = self.value_type();
        match val_type {
            TypeTag::Struct(struct_tag) => {
                let expect_type = T::struct_tag();
                //TODO define error code and rasie it to Move
                ensure!(
                    struct_tag.as_ref() == &expect_type,
                    "Expect type:{} but the state type:{}",
                    expect_type,
                    struct_tag
                );
                bcs::from_bytes(&self.value).map_err(Into::into)
            }
            _ => bail!("Expect type Object but the state type:{}", val_type),
        }
    }

    pub fn into_annotated_state<T: MoveResolver + ?Sized>(
        self,
        annotator: &MoveValueAnnotator<T>,
    ) -> Result<AnnotatedState> {
        let move_value = annotator.view_value(&self.value_type, &self.value)?;
        Ok(AnnotatedState::new(self, move_value))
    }
}

impl<T> From<T> for State
where
    T: MoveState,
{
    fn from(val: T) -> Self {
        let val_type = T::struct_tag();
        let val = bcs::to_bytes(&val).expect("Serialize MoveState to bcs should success");
        Self::new(val, TypeTag::Struct(Box::new(val_type)))
    }
}

#[derive(Debug, Clone)]
pub struct AnnotatedState {
    pub state: State,
    pub move_value: AnnotatedMoveValue,
}

impl AnnotatedState {
    pub fn new(state: State, move_value: AnnotatedMoveValue) -> Self {
        Self { state, move_value }
    }

    pub fn into_annotated_object(self) -> Result<AnnotatedObject> {
        ensure!(
            self.state.is_object(),
            "Expect state is a Object but found {:?}",
            self.state.value_type()
        );

        match self.move_value {
            AnnotatedMoveValue::Struct(annotated_move_object) => {
                AnnotatedObject::new_from_annotated_struct(annotated_move_object)
            }
            _ => bail!("Expect MoveStruct but found {:?}", self.move_value),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableTypeInfo {
    pub key_type: TypeTag,
}

impl TableTypeInfo {
    pub fn new(key_type: TypeTag) -> Self {
        Self { key_type }
    }
}

impl std::fmt::Display for TableTypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Table<{}>", self.key_type)
    }
}

/// Global State change set.
#[derive(Default, Clone, Debug)]
pub struct StateChangeSet {
    pub new_tables: BTreeMap<ObjectID, TableTypeInfo>,
    pub removed_tables: BTreeSet<ObjectID>,
    pub changes: BTreeMap<ObjectID, StateChange>,
}

/// A change of a single state.
#[derive(Clone, Debug)]
pub struct StateChange {
    //TODO should we keep the key's type here?
    pub entries: BTreeMap<Vec<u8>, Op<State>>,
}

/// A global state resolver which needs to be provided by the environment.
/// This allows to lookup data in remote storage.
/// If the handle is GLOBAL_OBJECT_STORAGE_HANDLE, it will get the data from the global state tree,
/// otherwise it will get the data from the table state tree.
/// The key can be an ObjectID or an arbitrary key of a table.
pub trait StateResolver {
    fn resolve_state(&self, handle: &ObjectID, key: &[u8]) -> Result<Option<State>, anyhow::Error>;
}
