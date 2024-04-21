// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::move_std::ascii::MoveAsciiString;
use crate::move_std::string::MoveString;
use crate::moveos_std::object::{AnnotatedObject, ObjectEntity, RawObject, GENESIS_STATE_ROOT};
use crate::moveos_std::object::{ObjectID, RootObjectEntity};
use anyhow::{bail, ensure, Result};
use core::str;
use move_core_types::language_storage::ModuleId;
use move_core_types::{
    account_address::AccountAddress,
    effects::Op,
    ident_str,
    identifier::{IdentStr, Identifier},
    language_storage::{StructTag, TypeTag},
    resolver::MoveResolver,
    u256::U256,
    value::{MoveStructLayout, MoveTypeLayout, MoveValue},
};
use move_resource_viewer::{AnnotatedMoveValue, MoveValueAnnotator};
use move_vm_types::values::{Struct, Value};
use primitive_types::H256;
use serde::ser::Error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;

/// `State` is represent state in MoveOS statedb, it can be a Move module or a Move Object or a Move resource or a Table value
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct State {
    /// the bytes of state
    pub value: Vec<u8>,
    /// the type of state
    pub value_type: TypeTag,
}

/// `KeyState` is represent key state
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct KeyState {
    /// the bytes of key state
    pub key: Vec<u8>,
    /// the type of key state
    pub key_type: TypeTag,
}

impl std::fmt::Debug for KeyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex_key = hex::encode(&self.key);
        write!(
            f,
            "key-type:{}, key: 0x{}",
            self.key_type.to_canonical_string(),
            hex_key
        )
    }
}

impl KeyState {
    pub fn new(key: Vec<u8>, key_type: TypeTag) -> Self {
        Self { key, key_type }
    }

    pub fn from_module_id(module_id: &ModuleId) -> Self {
        // The key is the moduleId string in bcs serialize format, not String::into_bytes.
        // bcs::to_bytes(&String) same as bcs::to_bytes(&MoveString)
        let key = bcs::to_bytes(&module_id.short_str_lossless())
            .expect("bcs to_bytes String must success.");
        //TODO should we use MoveAsciiString here? unify with resource key
        KeyState::new(key, MoveString::type_tag())
    }

    pub fn from_struct_tag(struct_tag: &StructTag) -> Self {
        // The resource key is struct_tag to_canonical_string in bcs serialize format string, not String::into_bytes.
        let key = bcs::to_bytes(&struct_tag.to_canonical_string())
            .expect("bcs to_bytes String must success.");
        KeyState::new(key, MoveAsciiString::type_tag())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        bcs::from_bytes(bytes)
            .map_err(|e| anyhow::anyhow!("Deserialize the KeyState error: {:?}", e))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bcs::to_bytes(self).map_err(|e| anyhow::anyhow!("Serialize the KeyState error: {:?}", e))
    }

    pub fn is_object_id(&self) -> bool {
        match &self.key_type {
            TypeTag::Struct(struct_tag) => ObjectID::struct_tag_match(struct_tag.as_ref()),
            _ => false,
        }
    }

    pub fn as_object_id(&self) -> Result<ObjectID> {
        ensure!(
            self.is_object_id(),
            "Expect key type is ObjectID but found {:?}",
            self.key_type
        );
        let object_id = ObjectID::from_bytes(&self.key)?;
        Ok(object_id)
    }

    pub fn into_annotated_state<T: MoveResolver + ?Sized>(
        self,
        annotator: &MoveValueAnnotator<T>,
    ) -> Result<AnnotatedKeyState> {
        let decoded_value = annotator.view_value(&self.key_type, &self.key)?;
        Ok(AnnotatedKeyState::new(self, decoded_value))
    }
}

impl std::fmt::Display for KeyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex_key = hex::encode(
            self.to_bytes()
                .map_err(|e| std::fmt::Error::custom(e.to_string()))?,
        );
        write!(f, "0x{}", hex_key)
    }
}

impl FromStr for KeyState {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key = hex::decode(s.strip_prefix("0x").unwrap_or(s))
            .map_err(|_| anyhow::anyhow!("Invalid key state str: {}", s))?;
        KeyState::from_bytes(key.as_slice())
    }
}

impl From<ObjectID> for KeyState {
    fn from(object_id: ObjectID) -> Self {
        object_id.to_key()
    }
}

impl From<ModuleId> for KeyState {
    fn from(module_id: ModuleId) -> Self {
        KeyState::from_module_id(&module_id)
    }
}

impl From<StructTag> for KeyState {
    fn from(tag: StructTag) -> Self {
        KeyState::from_struct_tag(&tag)
    }
}

//TODO find a better place for MoveType, MoveState and MoveStructState
/// The rust representation of a Move value
pub trait MoveType {
    fn type_tag() -> TypeTag;

    fn type_tag_match(type_tag: &TypeTag) -> bool {
        type_tag == &Self::type_tag()
    }
}

/// The rust representation of a Move Struct
/// This trait copy from `move_core_types::move_resource::MoveStructType`
/// For auto implement `MoveType` to `MoveStructType`
pub trait MoveStructType: MoveType {
    const ADDRESS: AccountAddress = move_core_types::language_storage::CORE_CODE_ADDRESS;
    const MODULE_NAME: &'static IdentStr;
    const STRUCT_NAME: &'static IdentStr;

    fn module_address() -> AccountAddress {
        Self::ADDRESS
    }

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

    /// Check the `struct_tag` argument is match Self::struct_tag
    fn struct_tag_match(struct_tag: &StructTag) -> bool {
        struct_tag == &Self::struct_tag()
    }

    /// Check the `struct_tag` argument is match Self::struct_tag, but ignore the type param.
    fn struct_tag_match_without_type_param(struct_tag: &StructTag) -> bool {
        struct_tag.address == Self::ADDRESS
            && struct_tag.module == Self::module_identifier()
            && struct_tag.name == Self::struct_identifier()
    }
}

fn type_layout_match(first_layout: &MoveTypeLayout, second_layout: &MoveTypeLayout) -> bool {
    match (first_layout, second_layout) {
        (MoveTypeLayout::Address, MoveTypeLayout::Address) => true,
        (MoveTypeLayout::Signer, MoveTypeLayout::Signer) => true,
        (MoveTypeLayout::Bool, MoveTypeLayout::Bool) => true,
        (MoveTypeLayout::U8, MoveTypeLayout::U8) => true,
        (MoveTypeLayout::U16, MoveTypeLayout::U16) => true,
        (MoveTypeLayout::U32, MoveTypeLayout::U32) => true,
        (MoveTypeLayout::U64, MoveTypeLayout::U64) => true,
        (MoveTypeLayout::U128, MoveTypeLayout::U128) => true,
        (MoveTypeLayout::U256, MoveTypeLayout::U256) => true,
        (
            MoveTypeLayout::Vector(first_inner_layout),
            MoveTypeLayout::Vector(second_inner_layout),
        ) => type_layout_match(first_inner_layout, second_inner_layout),
        (
            MoveTypeLayout::Struct(first_struct_layout),
            MoveTypeLayout::Struct(second_struct_layout),
        ) => {
            if first_struct_layout.fields().len() != second_struct_layout.fields().len() {
                false
            } else {
                first_struct_layout
                    .fields()
                    .iter()
                    .zip(second_struct_layout.fields().iter())
                    .all(|(first_field, second_field)| type_layout_match(first_field, second_field))
            }
        }
        (_, _) => false,
    }
}

/// The rust representation of a Move value state
pub trait MoveState: MoveType + DeserializeOwned + Serialize {
    fn type_layout() -> MoveTypeLayout;
    fn type_layout_match(other_type_layout: &MoveTypeLayout) -> bool {
        let self_layout = Self::type_layout();
        type_layout_match(&self_layout, other_type_layout)
    }
    fn from_bytes<T: AsRef<Vec<u8>>>(bytes: T) -> Result<Self>
    where
        Self: Sized,
    {
        bcs::from_bytes(bytes.as_ref())
            .map_err(|e| anyhow::anyhow!("Deserialize the MoveState error: {:?}", e))
    }
    fn to_bytes(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("Serialize the MoveState should success")
    }
    fn into_state(self) -> State {
        let value = self.to_bytes();
        State::new(value, Self::type_tag())
    }

    /// Convert the MoveState to MoveValue
    fn to_move_value(&self) -> MoveValue {
        let blob = self.to_bytes();
        MoveValue::simple_deserialize(&blob, &Self::type_layout())
            .expect("Deserialize the MoveValue from MoveState bytes should success")
    }

    /// Convert the MoveState to MoveRuntime Value
    fn to_runtime_value(&self) -> Value;

    /// Convert the MoveState from MoveRuntime Value
    fn from_runtime_value(value: Value) -> Result<Self>;
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

    fn to_runtime_value(&self) -> Value {
        Value::u8(*self)
    }

    fn from_runtime_value(value: Value) -> Result<Self> {
        Ok(value.value_as::<u8>()?)
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

    fn to_runtime_value(&self) -> Value {
        Value::u16(*self)
    }

    fn from_runtime_value(value: Value) -> Result<Self> {
        Ok(value.value_as::<u16>()?)
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

    fn to_runtime_value(&self) -> Value {
        Value::u32(*self)
    }

    fn from_runtime_value(value: Value) -> Result<Self> {
        Ok(value.value_as::<u32>()?)
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

    fn to_runtime_value(&self) -> Value {
        Value::u64(*self)
    }

    fn from_runtime_value(value: Value) -> Result<Self> {
        Ok(value.value_as::<u64>()?)
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

    fn to_runtime_value(&self) -> Value {
        Value::u128(*self)
    }

    fn from_runtime_value(value: Value) -> Result<Self> {
        Ok(value.value_as::<u128>()?)
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

    fn to_runtime_value(&self) -> Value {
        Value::u256(*self)
    }

    fn from_runtime_value(value: Value) -> Result<Self> {
        Ok(value.value_as::<U256>()?)
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

    fn to_runtime_value(&self) -> Value {
        Value::bool(*self)
    }

    fn from_runtime_value(value: Value) -> Result<Self> {
        Ok(value.value_as::<bool>()?)
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

    fn to_runtime_value(&self) -> Value {
        Value::address(*self)
    }

    fn from_runtime_value(value: Value) -> Result<Self> {
        Ok(value.value_as::<AccountAddress>()?)
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

    fn to_runtime_value(&self) -> Value {
        Value::struct_(S::to_runtime_value_struct(self))
    }

    fn from_runtime_value(value: Value) -> Result<Self> {
        let s = value.value_as::<Struct>()?;
        S::from_runtime_value_struct(s)
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

    fn to_runtime_value(&self) -> Value {
        let values: Vec<Value> = self.iter().map(MoveState::to_runtime_value).collect();
        Value::vector_for_testing_only(values)
    }

    fn from_runtime_value(value: Value) -> Result<Self> {
        let values = value.value_as::<Vec<Value>>()?;
        values
            .into_iter()
            .map(MoveState::from_runtime_value)
            .collect()
    }
}

/// A placeholder struct for unknown Move Struct
/// Sometimes we need a generic struct type, but we don't know the struct type
pub struct PlaceholderStruct;

impl MoveStructType for PlaceholderStruct {
    const ADDRESS: AccountAddress = AccountAddress::ZERO;
    const MODULE_NAME: &'static IdentStr = ident_str!("placeholder");
    const STRUCT_NAME: &'static IdentStr = ident_str!("PlaceholderStruct");

    fn type_params() -> Vec<TypeTag> {
        panic!("PlaceholderStruct should not be used as a type")
    }
}

/// Move State is a trait that is used to represent the state of a Move Resource in Rust
/// It is like the `MoveResource` in move_core_types
pub trait MoveStructState: MoveState + MoveStructType + DeserializeOwned + Serialize {
    fn struct_layout() -> MoveStructLayout;

    /// Convert the MoveState to MoveRuntime Value
    fn to_runtime_value_struct(&self) -> Struct {
        let blob = self.to_bytes();
        Struct::simple_deserialize(&blob, &Self::struct_layout())
            .expect("Deserialize the Move Runtime Value from MoveState bytes should success")
    }

    /// Convert the MoveState from MoveRuntime Value
    fn from_runtime_value_struct(value: Struct) -> Result<Self>
    where
        Self: Sized,
    {
        let blob = value
            .simple_serialize(&Self::struct_layout())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Serilaize the MoveState to bytes error: {:?}",
                    Self::type_tag()
                )
            })?;
        Self::from_bytes(blob)
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

pub trait MoveRuntimeValue {}

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
                if ObjectEntity::<PlaceholderStruct>::struct_tag_match_without_type_param(
                    struct_tag,
                ) {
                    let object_type_param = struct_tag
                        .type_params
                        .first()
                        .expect("The ObjectEntity<T> should have a type param");
                    match object_type_param {
                        TypeTag::Struct(struct_tag) => Some(struct_tag.as_ref().clone()),
                        _ => {
                            unreachable!("The ObjectEntity<T> should have a struct type param")
                        }
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// If the state is a Move resource, return the T's struct_tag
    /// Otherwise, return None
    pub fn get_resource_struct_tag(&self) -> Option<StructTag> {
        match self.value_type() {
            TypeTag::Struct(struct_tag) => Some(struct_tag.as_ref().clone()),
            _ => None,
        }
    }

    pub fn as_object<T>(&self) -> Result<ObjectEntity<T>>
    where
        T: MoveStructState,
    {
        self.cast::<ObjectEntity<T>>()
    }

    pub fn as_object_uncheck<T>(&self) -> Result<ObjectEntity<T>>
    where
        T: DeserializeOwned,
    {
        self.cast_unchecked::<ObjectEntity<T>>()
    }

    pub fn as_raw_object(&self) -> Result<RawObject> {
        let object_struct_tag = self.get_object_struct_tag().ok_or_else(|| {
            anyhow::anyhow!(
                "Expect type ObjectEntity<T> but the state type:{}",
                self.value_type
            )
        })?;
        RawObject::from_bytes(&self.value, object_struct_tag)
    }

    /// Case the state to T
    pub fn cast<T>(&self) -> Result<T>
    where
        T: MoveStructState,
    {
        let val_type = self.value_type();
        match val_type {
            TypeTag::Struct(struct_tag) => {
                //TODO define error code and rasie it to Move
                ensure!(
                    T::struct_tag_match(struct_tag),
                    "Expect type:{} but the state type:{}",
                    T::struct_tag(),
                    struct_tag
                );
                bcs::from_bytes(&self.value).map_err(Into::into)
            }
            _ => bail!("Expect type Object but the state type:{}", val_type),
        }
    }

    /// Directly cast the state to T without type check
    pub fn cast_unchecked<T: DeserializeOwned>(&self) -> Result<T> {
        bcs::from_bytes(&self.value).map_err(Into::into)
    }

    pub fn into_annotated_state<T: MoveResolver + ?Sized>(
        self,
        annotator: &MoveValueAnnotator<T>,
    ) -> Result<AnnotatedState> {
        let decoded_value = annotator.view_value(&self.value_type, &self.value)?;
        Ok(AnnotatedState::new(self, decoded_value))
    }
}

#[derive(Debug, Clone)]
pub struct AnnotatedState {
    pub state: State,
    pub decoded_value: AnnotatedMoveValue,
}

impl AnnotatedState {
    pub fn new(state: State, decoded_value: AnnotatedMoveValue) -> Self {
        Self {
            state,
            decoded_value,
        }
    }

    pub fn into_annotated_object(self) -> Result<AnnotatedObject> {
        ensure!(
            self.state.is_object(),
            "Expect state is a Object but found {:?}",
            self.state.value_type()
        );

        match self.decoded_value {
            AnnotatedMoveValue::Struct(annotated_move_object) => {
                AnnotatedObject::new_from_annotated_struct(annotated_move_object)
            }
            _ => bail!("Expect MoveStruct but found {:?}", self.decoded_value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnnotatedKeyState {
    pub state: KeyState,
    pub decoded_key: AnnotatedMoveValue,
}

impl AnnotatedKeyState {
    pub fn new(state: KeyState, decoded_key: AnnotatedMoveValue) -> Self {
        Self { state, decoded_key }
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

#[derive(Clone, Debug, Default)]
pub struct ObjectChange {
    pub op: Option<Op<State>>,
    pub fields: BTreeMap<KeyState, FieldChange>,
}

impl ObjectChange {
    pub fn new(op: Op<State>) -> Self {
        Self {
            op: Some(op),
            fields: BTreeMap::new(),
        }
    }

    pub fn new_empty() -> Self {
        Self {
            op: None,
            fields: BTreeMap::new(),
        }
    }

    pub fn add_field_change(&mut self, key: KeyState, field_change: FieldChange) {
        self.fields.insert(key, field_change);
    }

    pub fn add_field_changes(&mut self, field_states: Vec<FieldState>) {
        for v in field_states {
            self.fields.insert(v.key_state, v.field_change);
        }
    }

    pub fn get_field_change(&self, key: &KeyState) -> Option<&FieldChange> {
        self.fields.get(key)
    }

    pub fn get_field_change_mut(&mut self, key: &KeyState) -> Option<&mut FieldChange> {
        self.fields.get_mut(key)
    }

    pub fn remove_field_change(&mut self, key: &KeyState) -> Option<FieldChange> {
        self.fields.remove(key)
    }

    pub fn is_empty(&self) -> bool {
        self.op.is_none() && self.fields.is_empty()
    }

    /// just use for statedb dump and apply
    pub fn reset_state_root(&mut self, state_root: H256) -> Result<()> {
        if let Some(op) = self.op.clone() {
            match op.clone() {
                Op::New(state) | Op::Modify(state) => {
                    let mut obj = state.as_raw_object()?;
                    obj.state_root = AccountAddress::new(state_root.into());
                    if Op::New(state) == op {
                        self.op = Some(Op::New(obj.into_state()));
                    } else {
                        self.op = Some(Op::Modify(obj.into_state()));
                    }
                }
                Op::Delete => {}
            }
        };

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct NormalFieldChange {
    pub op: Op<State>,
}

#[derive(Clone, Debug)]
pub enum FieldChange {
    Object(ObjectChange),
    Normal(NormalFieldChange),
}

impl std::fmt::Display for FieldChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldChange::Object(object_change) => {
                write!(f, "{:?}", object_change)
            }
            FieldChange::Normal(normal_change) => {
                write!(f, "{:?}", normal_change)
            }
        }
    }
}

impl std::error::Error for FieldChange {}

impl FieldChange {
    pub fn new_normal(op: Op<State>) -> Self {
        FieldChange::Normal(NormalFieldChange { op })
    }

    pub fn new_object(object_change: ObjectChange) -> Self {
        FieldChange::Object(object_change)
    }

    pub fn into_object_change(self) -> Result<ObjectChange, FieldChange> {
        match self {
            FieldChange::Object(object_change) => Ok(object_change),
            _ => Err(self),
        }
    }

    pub fn into_normal_change(self) -> Result<NormalFieldChange, FieldChange> {
        match self {
            FieldChange::Normal(normal_change) => Ok(normal_change),
            _ => Err(self),
        }
    }
}

/// Global State change set.
#[derive(Clone, Debug)]
pub struct StateChangeSet {
    /// The state root before the changes
    pub state_root: H256,
    /// The root Object field size
    pub global_size: u64,
    pub changes: BTreeMap<ObjectID, ObjectChange>,
}

impl StateChangeSet {
    pub fn root_object(&self) -> RootObjectEntity {
        ObjectEntity::root_object(self.state_root, self.global_size)
    }
}

impl Default for StateChangeSet {
    fn default() -> Self {
        Self {
            state_root: *GENESIS_STATE_ROOT,
            global_size: 0,
            changes: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ObjectState {
    /// The state root of the Object
    pub state_root: H256,
    /// The Object field size
    pub size: u64,
    pub object_id: ObjectID,
    pub object_change: ObjectChange,
}

impl ObjectState {
    pub fn new(
        state_root: H256,
        size: u64,
        object_id: ObjectID,
        object_change: ObjectChange,
    ) -> ObjectState {
        Self {
            state_root,
            size,
            object_id,
            object_change,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FieldState {
    pub key_state: KeyState,
    pub field_change: FieldChange,
}

impl FieldState {
    pub fn new(key_state: KeyState, field_change: FieldChange) -> FieldState {
        Self {
            key_state,
            field_change,
        }
    }
}
