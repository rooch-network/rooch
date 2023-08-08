// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    object::{self, AnnotatedObject, Object, ObjectID, RawObject},
};
use anyhow::{bail, ensure, Result};
use move_core_types::{
    account_address::AccountAddress,
    effects::Op,
    identifier::{IdentStr, Identifier},
    language_storage::{StructTag, TypeTag},
    resolver::MoveResolver,
    u256::U256,
    value::{MoveStructLayout, MoveTypeLayout, MoveValue},
};
use move_resource_viewer::{AnnotatedMoveValue, MoveValueAnnotator};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use smt::UpdateSet;
use std::collections::{BTreeMap, BTreeSet};

/// `State` is represent state in MoveOS statedb, it can be a Move module or a Move Object or a Move resource or a Table value
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct State {
    /// the bytes of state
    pub value: Vec<u8>,
    /// the type of state
    pub value_type: TypeTag,
}

//TODO find a better place for MoveType, MoveState and MoveStructState
/// The rust representation of a Move value
pub trait MoveType {
    fn type_tag() -> TypeTag;
}

/// The rust representation of a Move Struct
/// This trait copy from `move_core_types::move_resource::MoveStructType`
/// For auto implement `MoveType` to `MoveStructType`
pub trait MoveStructType {
    const ADDRESS: AccountAddress = move_core_types::language_storage::CORE_CODE_ADDRESS;
    const MODULE_NAME: &'static IdentStr;
    const STRUCT_NAME: &'static IdentStr;

    fn module_identifier() -> Identifier {
        Self::MODULE_NAME.to_owned()
    }

    fn struct_identifier() -> Identifier {
        Self::STRUCT_NAME.to_owned()
    }

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            name: Self::struct_identifier(),
            module: Self::module_identifier(),
            type_params: Self::type_params(),
        }
    }
}

/// The rust representation of a Move value state
pub trait MoveState: MoveType + DeserializeOwned + Serialize {
    fn type_layout() -> MoveTypeLayout;
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        bcs::from_bytes(bytes)
            .map_err(|e| anyhow::anyhow!("Deserialize the MoveState error: {:?}", e))
    }
    fn to_bytes(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("Serialize the MoveState should success")
    }
    fn into_state(self) -> State {
        let value = self.to_bytes();
        State::new(value, Self::type_tag())
    }
    fn to_move_value(&self) -> MoveValue {
        let blob = self.to_bytes();
        MoveValue::simple_deserialize(&blob, &Self::type_layout())
            .expect("Deserialize the MoveValue from MoveState should success")
    }
}

impl MoveType for u8 {
    fn type_tag() -> TypeTag {
        TypeTag::U8
    }
}

impl MoveState for u8 {
    fn type_layout() -> MoveTypeLayout {
        MoveTypeLayout::U8
    }
}

impl MoveType for u16 {
    fn type_tag() -> TypeTag {
        TypeTag::U16
    }
}

impl MoveState for u16 {
    fn type_layout() -> MoveTypeLayout {
        MoveTypeLayout::U16
    }
}

impl MoveType for u32 {
    fn type_tag() -> TypeTag {
        TypeTag::U32
    }
}
impl MoveState for u32 {
    fn type_layout() -> MoveTypeLayout {
        MoveTypeLayout::U32
    }
}

impl MoveType for u64 {
    fn type_tag() -> TypeTag {
        TypeTag::U64
    }
}

impl MoveState for u64 {
    fn type_layout() -> MoveTypeLayout {
        MoveTypeLayout::U64
    }
}

impl MoveType for u128 {
    fn type_tag() -> TypeTag {
        TypeTag::U128
    }
}

impl MoveState for u128 {
    fn type_layout() -> MoveTypeLayout {
        MoveTypeLayout::U128
    }
}

impl MoveType for U256 {
    fn type_tag() -> TypeTag {
        TypeTag::U256
    }
}

impl MoveState for U256 {
    fn type_layout() -> MoveTypeLayout {
        MoveTypeLayout::U256
    }
}

impl MoveType for bool {
    fn type_tag() -> TypeTag {
        TypeTag::Bool
    }
}

impl MoveState for bool {
    fn type_layout() -> MoveTypeLayout {
        MoveTypeLayout::Bool
    }
}

impl MoveType for AccountAddress {
    fn type_tag() -> TypeTag {
        TypeTag::Address
    }
}

impl MoveState for AccountAddress {
    fn type_layout() -> MoveTypeLayout {
        MoveTypeLayout::Address
    }
}

impl<S> MoveType for S
where
    S: MoveStructType,
{
    fn type_tag() -> TypeTag {
        TypeTag::Struct(Box::new(Self::struct_tag()))
    }
}

impl<S> MoveState for S
where
    S: MoveStructState,
{
    fn type_layout() -> MoveTypeLayout {
        MoveTypeLayout::Struct(Self::struct_layout())
    }
}

impl<S> MoveType for Vec<S>
where
    S: MoveType,
{
    fn type_tag() -> TypeTag {
        TypeTag::Vector(Box::new(S::type_tag()))
    }
}

impl<S> MoveState for Vec<S>
where
    S: MoveState,
{
    fn type_layout() -> MoveTypeLayout {
        MoveTypeLayout::Vector(Box::new(S::type_layout()))
    }
}

/// Move State is a trait that is used to represent the state of a Move Resource in Rust
/// It is like the `MoveResource` in move_core_types
pub trait MoveStructState: MoveStructType + DeserializeOwned + Serialize {
    fn type_layout() -> MoveTypeLayout {
        MoveTypeLayout::Struct(Self::struct_layout())
    }
    fn struct_layout() -> MoveStructLayout;
    fn type_match(type_tag: &StructTag) -> bool {
        type_tag == &Self::struct_tag()
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        bcs::from_bytes(bytes)
            .map_err(|e| anyhow::anyhow!("Deserialize the MoveState error: {:?}", e))
    }
    fn to_bytes(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("Serialize the MoveState should success")
    }
    fn into_state(self) -> State {
        let value = self.to_bytes();
        State::new(value, TypeTag::Struct(Box::new(Self::struct_tag())))
    }
}

impl<T> From<T> for State
where
    T: MoveStructState,
{
    fn from(state: T) -> Self {
        state.into_state()
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
                if struct_tag.address == MOVEOS_STD_ADDRESS
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
        T: MoveStructState,
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
        T: MoveStructState,
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
    pub changes: BTreeMap<ObjectID, TableChange>,
}

/// A change of a single table.
#[derive(Default, Clone, Debug)]
pub struct TableChange {
    //TODO should we keep the key's type here?
    pub entries: BTreeMap<Vec<u8>, Op<State>>,
}

/// StateSet is represent state dump result. Not include events and other stores
#[derive(Clone, Debug, Default)]
pub struct StateSet {
    pub state_sets: BTreeMap<ObjectID, UpdateSet<Vec<u8>, State>>,
}

impl StateSet {
    pub fn insert(
        &mut self,
        k: ObjectID,
        v: UpdateSet<Vec<u8>, State>,
    ) -> Option<UpdateSet<Vec<u8>, State>> {
        self.state_sets.insert(k, v)
    }
}
