// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::h256;
use crate::move_std::string::MoveString;
use crate::moveos_std::module_store::ModuleStore;
use crate::moveos_std::object::{
    AnnotatedObject, DynamicField, ObjectEntity, ObjectID, ObjectMeta, RawData, RawObject, Root,
    GENESIS_STATE_ROOT,
};
use crate::moveos_std::timestamp::Timestamp;
use anyhow::{bail, ensure, Result};
use core::str;
use hex::FromHex;
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
use move_resource_viewer::{AnnotatedMoveStruct, MoveValueAnnotator};
use move_vm_types::values::{Struct, Value};
use primitive_types::H256;
use serde::{
    de::DeserializeOwned, de::Error as _, Deserialize, Deserializer, Serialize, Serializer,
};
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;
/// `ObjectState` is represent state in MoveOS statedb
/// It can be DynamicField  or user defined Move Struct
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ObjectState {
    pub metadata: ObjectMeta,
    pub value: Vec<u8>,
}

/// `FieldKey` is represent field key in statedb, it is a hash of (key|key_type)
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub struct FieldKey(pub [u8; FieldKey::LENGTH]);

impl FieldKey {
    pub const LENGTH: usize = AccountAddress::LENGTH;

    pub fn new(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    /// Derive a dynamic field key from the key
    pub fn derive<K>(key: &K) -> Result<Self>
    where
        K: ?Sized + Serialize + MoveType + Debug,
    {
        let key_type_tag = K::type_tag();
        let k_tag_str = key_type_tag.to_canonical_string();
        tracing::trace!(
            "Deriving dynamic field key for key={:?}, key_type_tag={:?}",
            key,
            k_tag_str,
        );

        // hash(key || key_type_tag)
        let mut buffer = bcs::to_bytes(key)?;
        buffer.extend_from_slice(k_tag_str.as_bytes());
        let hash = h256::sha3_256_of(&buffer);
        Ok(FieldKey(hash.0))
    }

    pub fn derive_from_string(s: &str) -> Self {
        Self::derive(&MoveString::from(s))
            .expect("Derive dynamic field key with MoveString should not fail")
    }

    pub fn derive_from_address(address: &AccountAddress) -> Self {
        Self::derive(address).expect("Derive dynamic field key with AccountAddress should not fail")
    }

    /// The module of Package use the module name to derive the key
    pub fn derive_module_key(module_name: &IdentStr) -> Self {
        // bcs::to_bytes(&String) same as bcs::to_bytes(&MoveString)
        let module_name_str = MoveString::from(module_name);
        Self::derive(&module_name_str)
            .expect("Derive dynamic field key with MoveString should not fail")
    }

    /// The resource of Account use the StructTag string to derive the key
    pub fn derive_resource_key(struct_tag: &StructTag) -> Self {
        let struct_tag_str = MoveString::from(struct_tag.to_canonical_string());
        Self::derive(&struct_tag_str)
            .expect("Derive dynamic field key with MoveString should not fail")
    }

    pub fn from_hex(hex: &str) -> Result<Self> {
        Ok(<[u8; Self::LENGTH]>::from_hex(hex).map(FieldKey::new)?)
    }

    pub fn from_hex_literal(hex: &str) -> Result<Self> {
        //We use the AccountAddress::from_hex_literal for support the short hex format.
        Ok(AccountAddress::from_hex_literal(hex)?.into())
    }

    pub fn to_hex(&self) -> String {
        format!("{:x}", self)
    }

    pub fn to_hex_literal(&self) -> String {
        format!("{:#x}", self)
    }

    pub fn random() -> Self {
        AccountAddress::random().into()
    }
}

impl AsRef<[u8]> for FieldKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::ops::Deref for FieldKey {
    type Target = [u8; Self::LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for FieldKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        //append 0x prefix
        write!(f, "{:#x}", self)
    }
}

impl fmt::Debug for FieldKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self)
    }
}

impl fmt::LowerHex for FieldKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }

        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl fmt::UpperHex for FieldKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }

        for byte in &self.0 {
            write!(f, "{:02X}", byte)?;
        }

        Ok(())
    }
}

impl From<[u8; FieldKey::LENGTH]> for FieldKey {
    fn from(bytes: [u8; FieldKey::LENGTH]) -> Self {
        Self::new(bytes)
    }
}

impl From<AccountAddress> for FieldKey {
    fn from(address: AccountAddress) -> Self {
        Self(address.into_bytes())
    }
}

impl From<FieldKey> for AccountAddress {
    fn from(field_key: FieldKey) -> Self {
        AccountAddress::from(field_key.0)
    }
}

impl From<H256> for FieldKey {
    fn from(hash: H256) -> Self {
        Self(hash.0)
    }
}

impl From<FieldKey> for H256 {
    fn from(field_key: FieldKey) -> Self {
        Self(field_key.0)
    }
}

impl FromStr for FieldKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        Self::from_hex_literal(s)
    }
}

impl<'de> Deserialize<'de> for FieldKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            FieldKey::from_str(&s).map_err(D::Error::custom)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "FieldKey")]
            struct Value([u8; FieldKey::LENGTH]);

            let value = Value::deserialize(deserializer)?;
            Ok(FieldKey::new(value.0))
        }
    }
}

impl Serialize for FieldKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.to_hex_literal().serialize(serializer)
        } else {
            // See comment in deserialize.
            serializer.serialize_newtype_struct("FieldKey", &self.0)
        }
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

impl MoveStructType for String {
    const ADDRESS: AccountAddress = MoveString::ADDRESS;
    const MODULE_NAME: &'static IdentStr = MoveString::MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = MoveString::STRUCT_NAME;
}

impl MoveStructState for String {
    fn struct_layout() -> MoveStructLayout {
        MoveString::struct_layout()
    }
}

impl MoveType for str {
    fn type_tag() -> TypeTag {
        MoveString::type_tag()
    }
}

impl MoveType for [u8] {
    fn type_tag() -> TypeTag {
        TypeTag::Vector(Box::new(TypeTag::U8))
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
                    "Serialize the MoveState to bytes error: {:?}",
                    Self::type_tag()
                )
            })?;
        Self::from_bytes(blob)
    }
}

impl ObjectState {
    pub fn new(metadata: ObjectMeta, value: Vec<u8>) -> Self {
        Self { metadata, value }
    }

    pub fn new_with_struct<S>(metadata: ObjectMeta, value: S) -> Result<Self>
    where
        S: MoveStructState,
    {
        let value_bytes = value.to_bytes();
        ensure!(
            metadata.match_struct_type(&S::struct_tag()),
            "Expect type:{} but the state type:{}",
            S::struct_tag(),
            metadata.object_struct_tag()
        );
        Ok(Self::new(metadata, value_bytes))
    }

    pub fn new_root(root_metadata: ObjectMeta) -> Self {
        Self::new_with_struct(root_metadata, Root::default())
            .expect("Create Root Object should success")
    }

    /// Create Timestamp Object
    pub fn new_timestamp(timestamp: Timestamp) -> Self {
        let id = Timestamp::object_id();
        let mut metadata = ObjectMeta::genesis_meta(id, Timestamp::type_tag());
        metadata.to_shared();
        Self::new_with_struct(metadata, timestamp).expect("Create Timestamp Object should success")
    }

    /// Create Genesis ModuleStore Object
    pub fn genesis_module_store() -> Self {
        let id = ModuleStore::object_id();
        let mut metadata = ObjectMeta::genesis_meta(id, ModuleStore::type_tag());
        metadata.to_shared();
        Self::new_with_struct(metadata, ModuleStore::default())
            .expect("Create ModuleStore Object should success")
    }

    pub fn id(&self) -> &ObjectID {
        &self.metadata.id
    }

    pub fn object_type(&self) -> &TypeTag {
        &self.metadata.object_type
    }

    pub fn object_struct_tag(&self) -> &StructTag {
        self.metadata.object_struct_tag()
    }

    pub fn flag(&self) -> u8 {
        self.metadata.flag
    }

    pub fn state_root(&self) -> H256 {
        self.metadata.state_root()
    }

    pub fn size(&self) -> u64 {
        self.metadata.size
    }

    pub fn created_at(&self) -> u64 {
        self.metadata.created_at
    }

    pub fn updated_at(&self) -> u64 {
        self.metadata.updated_at
    }

    pub fn match_type(&self, type_tag: &TypeTag) -> bool {
        self.metadata.match_type(type_tag)
    }

    pub fn match_struct_type(&self, type_tag: &StructTag) -> bool {
        self.metadata.match_struct_type(type_tag)
    }

    pub fn match_dynamic_field_type(&self, name_type: TypeTag, value_type: TypeTag) -> bool {
        self.metadata
            .match_dynamic_field_type(name_type, value_type)
    }

    pub fn is_dynamic_field(&self) -> bool {
        self.metadata.is_dynamic_field()
    }

    pub fn is_object(&self) -> bool {
        self.metadata.is_object()
    }

    pub fn is_frozen(&self) -> bool {
        self.metadata.is_frozen()
    }

    pub fn is_shared(&self) -> bool {
        self.metadata.is_shared()
    }

    pub fn owner(&self) -> AccountAddress {
        self.metadata.owner
    }

    pub fn update_state_root(&mut self, new_state_root: H256) {
        self.metadata.update_state_root(new_state_root);
    }

    pub fn value_as<T>(&self) -> Result<T>
    where
        T: MoveStructState,
    {
        let val_type = self.object_type();
        match val_type {
            TypeTag::Struct(struct_tag) => {
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

    pub fn value_as_df<N, V>(&self) -> Result<DynamicField<N, V>>
    where
        N: MoveState,
        V: MoveState,
    {
        self.value_as::<DynamicField<N, V>>()
    }

    pub fn value_as_uncheck<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        bcs::from_bytes(&self.value).map_err(Into::into)
    }

    pub fn into_object<T>(self) -> Result<ObjectEntity<T>>
    where
        T: MoveStructState,
    {
        let value = self.value_as()?;
        Ok(ObjectEntity::new_with_object_meta(self.metadata, value))
    }

    pub fn into_object_uncheck<T>(self) -> Result<ObjectEntity<T>>
    where
        T: DeserializeOwned,
    {
        let value = self.value_as_uncheck::<T>()?;
        Ok(ObjectEntity::new_with_object_meta(self.metadata, value))
    }

    pub fn into_raw_object(self) -> Result<RawObject> {
        let object_struct_tag = self.object_struct_tag().clone();
        Ok(RawObject::new_with_object_meta(
            self.metadata,
            RawData {
                struct_tag: object_struct_tag,
                value: self.value,
            },
        ))
    }

    pub fn into_inner(self) -> (ObjectMeta, Vec<u8>) {
        (self.metadata, self.value)
    }

    pub fn into_annotated_state<T: MoveResolver + ?Sized>(
        self,
        annotator: &MoveValueAnnotator<T>,
    ) -> Result<AnnotatedState> {
        let decoded_value = annotator
            .view_resource(self.object_struct_tag(), &self.value)
            .map_err(|e| anyhow::anyhow!("Annotate the MoveValue error: {:?}", e))?;
        Ok(AnnotatedState::new(self, decoded_value))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        bcs::from_bytes(bytes).map_err(|e| anyhow::anyhow!("Deserialize the State error: {:?}", e))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bcs::to_bytes(self).map_err(|e| anyhow::anyhow!("Serialize the State error: {:?}", e))
    }
}

impl std::fmt::Display for ObjectState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex_state = hex::encode(self.to_bytes().map_err(|_e| std::fmt::Error)?);
        write!(f, "0x{}", hex_state)
    }
}

impl FromStr for ObjectState {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let state = hex::decode(s.strip_prefix("0x").unwrap_or(s))
            .map_err(|_| anyhow::anyhow!("Invalid state str: {}", s))?;
        ObjectState::from_bytes(state.as_slice())
    }
}

#[derive(Debug, Clone)]
pub struct AnnotatedState {
    pub metadata: ObjectMeta,
    pub value: Vec<u8>,
    pub decoded_value: AnnotatedMoveStruct,
}

impl AnnotatedState {
    pub fn new(state: ObjectState, decoded_value: AnnotatedMoveStruct) -> Self {
        Self {
            metadata: state.metadata,
            value: state.value,
            decoded_value,
        }
    }

    pub fn into_inner(self) -> (ObjectMeta, Vec<u8>, AnnotatedMoveStruct) {
        (self.metadata, self.value, self.decoded_value)
    }

    pub fn into_annotated_object(self) -> Result<AnnotatedObject> {
        Ok(AnnotatedObject::new_with_object_meta(
            self.metadata,
            self.decoded_value,
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectChange {
    pub metadata: ObjectMeta,
    #[serde(with = "op_serde")]
    pub value: Option<Op<Vec<u8>>>,
    pub fields: BTreeMap<FieldKey, ObjectChange>,
}

impl ObjectChange {
    pub fn new(metadata: ObjectMeta, value: Op<Vec<u8>>) -> Self {
        Self {
            metadata,
            value: Some(value),
            fields: BTreeMap::new(),
        }
    }

    pub fn meta(metadata: ObjectMeta) -> Self {
        Self {
            metadata,
            value: None,
            fields: BTreeMap::new(),
        }
    }

    pub fn new_object(object: ObjectState) -> Self {
        let (metadata, value) = object.into_inner();
        Self::new(metadata, Op::New(value))
    }

    pub fn add_field_change(&mut self, field_change: ObjectChange) -> Result<()> {
        ensure!(
            field_change.metadata.id.parent() == Some(self.metadata.id.clone()),
            "FieldChange id parent not match with ObjectChange id: {:?} != {:?}",
            field_change.metadata.id,
            self.metadata.id
        );
        match &field_change.value {
            Some(op) => match op {
                Op::New(_) => {
                    self.metadata.size += 1;
                }
                Op::Delete => {
                    self.metadata.size -= 1;
                }
                Op::Modify(_) => {}
            },
            None => {}
        }
        let key = field_change.metadata.id.field_key();
        self.fields.insert(key, field_change);
        Ok(())
    }

    pub fn update_state_root(&mut self, new_state_root: H256) {
        self.metadata.update_state_root(new_state_root);
    }
}

/// Global State change set.
/// The state_root in the ObjectChange is the state_root before the changes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateChangeSet {
    /// The state root of the root Object
    pub state_root: H256,
    /// The field size of the root Object
    pub global_size: u64,
    pub changes: BTreeMap<FieldKey, ObjectChange>,
}

impl StateChangeSet {
    pub fn new(state_root: H256, global_size: u64) -> Self {
        Self {
            state_root,
            global_size,
            changes: BTreeMap::new(),
        }
    }

    pub fn root_metadata(&self) -> ObjectMeta {
        ObjectMeta::root_metadata(self.state_root, self.global_size)
    }

    pub fn update_state_root(&mut self, new_state_root: H256) {
        self.state_root = new_state_root;
    }

    pub fn add_change(&mut self, change: ObjectChange) -> Result<()> {
        let id = change.metadata.id.clone();
        let parent = id.parent().expect("No root ObjectChange have parent");
        if parent.is_root() {
            let key = id.field_key();
            match &change.value {
                Some(op) => match op {
                    Op::New(_) => self.global_size += 1,
                    Op::Delete => self.global_size -= 1,
                    Op::Modify(_) => {}
                },
                None => {}
            }
            self.changes.insert(key, change);
        } else {
            let parent_key = parent.field_key();
            let parent_change = self
                .changes
                .get_mut(&parent_key)
                .ok_or_else(|| anyhow::anyhow!("Parent ObjectChange not found for id: {:?}", id))?;
            parent_change.add_field_change(change)?;
        }
        Ok(())
    }

    pub fn add_new_object(&mut self, object: ObjectState) -> Result<()> {
        let (metadata, value) = object.into_inner();
        let change = ObjectChange::new(metadata, Op::New(value));
        self.add_change(change)
    }
}

impl Default for StateChangeSet {
    fn default() -> Self {
        Self {
            state_root: *GENESIS_STATE_ROOT,
            global_size: 0,
            changes: BTreeMap::new(),
        }
    }
}

mod op_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
    pub enum SerializableOp<T> {
        New(T),
        Modify(T),
        Delete,
    }

    impl<T> From<Op<T>> for SerializableOp<T> {
        fn from(op: Op<T>) -> Self {
            match op {
                Op::New(value) => SerializableOp::New(value),
                Op::Modify(value) => SerializableOp::Modify(value),
                Op::Delete => SerializableOp::Delete,
            }
        }
    }

    impl<T> From<SerializableOp<T>> for Op<T> {
        fn from(op: SerializableOp<T>) -> Self {
            match op {
                SerializableOp::New(value) => Op::New(value),
                SerializableOp::Modify(value) => Op::Modify(value),
                SerializableOp::Delete => Op::Delete,
            }
        }
    }

    pub fn serialize<S, T>(option_op: &Option<Op<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize + Clone,
    {
        let op = option_op.as_ref().cloned();
        op.map(SerializableOp::from).serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<Op<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        let op = Option::<SerializableOp<T>>::deserialize(deserializer)
            .map_err(|e| D::Error::custom(format!("Deserialize the Op<T> error: {:?}", e)))?;
        Ok(op.map(Op::from))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field_key_derive_test<K>(k: &K, expect_result: &str)
    where
        K: ?Sized + Serialize + MoveType + Debug,
    {
        let key = FieldKey::derive(k).unwrap();
        //println!("key: {}", key);
        assert_eq!(
            expect_result,
            key.to_hex_literal(),
            "FieldKey derive from {:?} should be {}",
            k,
            expect_result
        );
    }

    //Make sure the FieldKey derive function result is same with it in Move
    #[test]
    fn test_field_key_derive_cases() {
        //test vector
        field_key_derive_test(
            b"1".as_slice(),
            "0x7301c6d045ed0df28fa129f5a825b210c8300eb0f44bb302e8a54b5eebeae13f",
        );
        //test string
        field_key_derive_test(
            "1",
            "0xc62df9a91eae549c2ff104f121549251c748185d0a21d5018c87db4be47fd191",
        );
        //test u8
        field_key_derive_test(
            &1u8,
            "0x988ba0cd547556c2014c5e718b15fce912b95aa39db882de598b6ea841cde194",
        );
        //test u64
        field_key_derive_test(
            &1u64,
            "0x7eb4036673c8611e43c3eff1202446612f22a4b3bac92b7e14c0562ade5f1a3f",
        );
        //test address
        field_key_derive_test(
            &AccountAddress::ONE,
            "0x07d29b5cffb95d39f98baed1a973e676891bc9d379022aba6f4a2e4912a5e552",
        );
    }
}
