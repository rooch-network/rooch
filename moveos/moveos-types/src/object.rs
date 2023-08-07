// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    h256,
    state::{MoveStructState, MoveStructType, State},
};
use anyhow::{bail, ensure, Result};
use fastcrypto::encoding::Hex;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::{StructTag, TypeTag},
    value::{MoveStructLayout, MoveTypeLayout},
};
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Specific Table Object ID associated with an address
#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NamedTableID {
    Resource(AccountAddress),
    Module(AccountAddress),
}

impl NamedTableID {
    const RESOURCE_TABLE_INDEX: u64 = 0;
    const MODULE_TABLE_INDEX: u64 = 1;

    pub fn to_object_id(self) -> ObjectID {
        self.into()
    }

    pub fn account(&self) -> AccountAddress {
        match self {
            NamedTableID::Resource(addr) => *addr,
            NamedTableID::Module(addr) => *addr,
        }
    }

    pub fn table_index(&self) -> u64 {
        match self {
            NamedTableID::Resource(_) => Self::RESOURCE_TABLE_INDEX,
            NamedTableID::Module(_) => Self::MODULE_TABLE_INDEX,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord, Hash, JsonSchema)]
pub struct ObjectID(#[schemars(with = "Hex")] AccountAddress);

impl ObjectID {
    const LENGTH: usize = h256::LENGTH;

    /// Creates a new ObjectID
    pub const fn new(obj_id: [u8; Self::LENGTH]) -> Self {
        Self(AccountAddress::new(obj_id))
    }

    /// Hex address: 0x0
    pub const ZERO: Self = Self::new([0u8; Self::LENGTH]);

    /// Hex address: 0x1
    pub const ONE: Self = Self::get_hex_object_id_one();

    /// Hex address: 0x2
    pub const TWO: Self = Self::get_hex_object_id_two();

    const fn get_hex_object_id_one() -> Self {
        let mut addr = [0u8; AccountAddress::LENGTH];
        addr[AccountAddress::LENGTH - 1] = 1u8;
        Self::new(addr)
    }

    const fn get_hex_object_id_two() -> Self {
        let mut addr = [0u8; AccountAddress::LENGTH];
        addr[AccountAddress::LENGTH - 1] = 2u8;
        Self::new(addr)
    }

    /// Create an ObjectID from transaction hash digest and `creation_num`.
    /// Caller is responsible for ensuring that hash is unique and
    /// `creation_num` is fresh
    pub fn derive_id(tx_hash: Vec<u8>, creation_num: u64) -> Self {
        let mut buffer = tx_hash;
        buffer.extend(creation_num.to_le_bytes());
        Self::new(h256::sha3_256_of(&buffer).into())
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        <[u8; Self::LENGTH]>::try_from(bytes.as_ref())
            .map_err(|_| anyhow::anyhow!("Invalid ObjectID bytes, length:{}", bytes.as_ref().len()))
            .map(ObjectID::from)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl std::fmt::Display for ObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The AccountAddress display has no prefix, so we add it here
        write!(f, "0x{}", self.0)
    }
}

impl MoveStructType for ObjectID {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("object_id");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ObjectID");
}

impl MoveStructState for ObjectID {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Address])
    }
}

impl Serialize for ObjectID {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(self.to_string().as_str())
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for ObjectID {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Ok(ObjectID::from_str(s.as_str()).map_err(serde::de::Error::custom)?)
        } else {
            Ok(ObjectID(AccountAddress::deserialize(deserializer)?))
        }
    }
}

/// Try to convert moveos_std::object_id::ObjectID' MoveValue to ObjectID
impl TryFrom<AnnotatedMoveValue> for ObjectID {
    type Error = anyhow::Error;

    fn try_from(value: AnnotatedMoveValue) -> Result<Self, Self::Error> {
        match value {
            AnnotatedMoveValue::Struct(annotated_move_struct) => {
                ObjectID::try_from(annotated_move_struct)
            }
            _ => Err(anyhow::anyhow!("Invalid ObjectID")),
        }
    }
}

impl TryFrom<AnnotatedMoveStruct> for ObjectID {
    type Error = anyhow::Error;

    fn try_from(value: AnnotatedMoveStruct) -> Result<Self, Self::Error> {
        let mut annotated_move_struct = value;
        let (field_name, field_value) = annotated_move_struct
            .value
            .pop()
            .ok_or_else(|| anyhow::anyhow!("Invalid ObjectID"))?;
        debug_assert!(field_name.as_str() == "id");
        let account_address = match field_value {
            AnnotatedMoveValue::Address(account_address) => account_address,
            _ => return Err(anyhow::anyhow!("Invalid ObjectID")),
        };
        Ok(ObjectID(account_address))
    }
}

impl From<[u8; ObjectID::LENGTH]> for ObjectID {
    fn from(bytes: [u8; ObjectID::LENGTH]) -> Self {
        Self::new(bytes)
    }
}

impl From<AccountAddress> for ObjectID {
    fn from(address: AccountAddress) -> Self {
        ObjectID(address)
    }
}

impl From<ObjectID> for AccountAddress {
    fn from(object_id: ObjectID) -> Self {
        AccountAddress::new(object_id.0.into())
    }
}

impl From<NamedTableID> for ObjectID {
    fn from(named_object_id: NamedTableID) -> Self {
        ObjectID::derive_id(
            named_object_id.account().to_vec(),
            named_object_id.table_index(),
        )
    }
}

impl FromStr for ObjectID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let address = AccountAddress::from_hex_literal(s)
            .map_err(|_e| anyhow::anyhow!("Invalid ObjectID:{}|", s))?;
        Ok(ObjectID::from(address))
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct AccountStorage {
    pub resources: ObjectID,
    pub modules: ObjectID,
}

impl AccountStorage {
    pub fn new(account: AccountAddress) -> Self {
        let resources = NamedTableID::Resource(account).to_object_id();
        let modules = NamedTableID::Module(account).to_object_id();
        AccountStorage { resources, modules }
    }
}

impl MoveStructType for AccountStorage {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("account_storage");
    const STRUCT_NAME: &'static IdentStr = ident_str!("AccountStorage");

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveStructState for AccountStorage {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::Struct(ObjectID::struct_layout()),
            MoveTypeLayout::Struct(ObjectID::struct_layout()),
        ])
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct TableInfo {
    //TODO use u256?
    pub state_root: AccountAddress,
    //TODO keep Table Key TypeTag at here
}

impl TableInfo {
    pub fn new(state_root: AccountAddress) -> Self {
        TableInfo { state_root }
    }
}

pub const TABLE_INFO_MODULE_NAME: &IdentStr = ident_str!("raw_table");
pub const TABLE_INFO_STRUCT_NAME: &IdentStr = ident_str!("TableInfo");

impl MoveStructType for TableInfo {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = TABLE_INFO_MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = TABLE_INFO_STRUCT_NAME;

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveStructState for TableInfo {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Address])
    }
}

pub type TableObject = Object<TableInfo>;
pub type AccountStorageObject = Object<AccountStorage>;

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct Object<T> {
    pub id: ObjectID,
    pub owner: AccountAddress,
    //#[serde(flatten)]
    pub value: T,
}

impl<T> Object<T> {
    pub fn new(id: ObjectID, owner: AccountAddress, value: T) -> Object<T> {
        Self { id, owner, value }
    }
}

impl<T> Object<T>
where
    T: MoveStructState,
{
    pub fn to_bytes(&self) -> Vec<u8> {
        bcs::to_bytes(self).unwrap()
    }

    pub fn to_raw(&self) -> RawObject {
        RawObject {
            id: self.id,
            owner: self.owner,
            value: RawData {
                struct_tag: T::struct_tag(),
                value: bcs::to_bytes(&self.value).expect("MoveState to bcs should success"),
            },
        }
    }
}

impl<T> From<Object<T>> for RawObject
where
    T: MoveStructState,
{
    fn from(object: Object<T>) -> Self {
        object.to_raw()
    }
}

impl Object<TableInfo> {
    pub fn new_table_object(id: ObjectID, value: TableInfo) -> TableObject {
        Self {
            id,
            //TODO table should have a owner?
            owner: AccountAddress::ZERO,
            value,
        }
    }
}

impl Object<AccountStorage> {
    pub fn new_account_storage_object(account: AccountAddress) -> AccountStorageObject {
        Self {
            id: ObjectID::from(account),
            owner: account,
            value: AccountStorage::new(account),
        }
    }
}

pub const OBJECT_MODULE_NAME: &IdentStr = ident_str!("object");
pub const OBJECT_STRUCT_NAME: &IdentStr = ident_str!("Object");

impl<T> MoveStructType for Object<T>
where
    T: MoveStructType,
{
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = OBJECT_MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = OBJECT_STRUCT_NAME;

    fn type_params() -> Vec<TypeTag> {
        vec![TypeTag::Struct(Box::new(T::struct_tag()))]
    }
}

impl<T> MoveStructState for Object<T>
where
    T: MoveStructState,
{
    /// Return the layout of the Object in Move
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::Struct(ObjectID::struct_layout()),
            MoveTypeLayout::Address,
            MoveTypeLayout::Struct(T::struct_layout()),
        ])
    }
}

pub type RawObject = Object<RawData>;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct RawData {
    pub struct_tag: StructTag,
    pub value: Vec<u8>,
}

impl RawObject {
    pub fn from_bytes(bytes: &[u8], struct_tag: StructTag) -> Result<Self> {
        ensure!(
            bytes.len() > ObjectID::LENGTH + AccountAddress::LENGTH,
            "Invalid bytes length"
        );

        let id: ObjectID = bcs::from_bytes(&bytes[..ObjectID::LENGTH])?;
        let owner: AccountAddress = bcs::from_bytes(
            &bytes[AccountAddress::LENGTH..ObjectID::LENGTH + AccountAddress::LENGTH],
        )?;
        let value = bytes[ObjectID::LENGTH + AccountAddress::LENGTH..].to_vec();
        Ok(RawObject {
            id,
            owner,
            value: RawData { struct_tag, value },
        })
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(bcs::to_bytes(&self.id).unwrap());
        bytes.extend(bcs::to_bytes(&self.owner).unwrap());
        bytes.extend_from_slice(&self.value.value);
        bytes
    }
}

impl TryFrom<State> for RawObject {
    type Error = anyhow::Error;

    fn try_from(state: State) -> Result<Self> {
        state.as_raw_object()
    }
}

pub type AnnotatedObject = Object<AnnotatedMoveStruct>;

impl AnnotatedObject {
    pub fn new_annotated_object(
        id: ObjectID,
        owner: AccountAddress,
        value: AnnotatedMoveStruct,
    ) -> Self {
        Self::new(id, owner, value)
    }

    /// Create a new AnnotatedObject from a AnnotatedMoveStruct
    /// The MoveStruct is Object<T> in Move, not the T's value
    pub fn new_from_annotated_struct(object_struct: AnnotatedMoveStruct) -> Result<Self> {
        let mut fields = object_struct.value.into_iter();
        let object_id = ObjectID::try_from(fields.next().expect("Object should have id").1)?;
        let owner = match fields.next().expect("Object should have owner") {
            (field_name, AnnotatedMoveValue::Address(filed_value)) => {
                debug_assert!(
                    field_name.as_str() == "owner",
                    "Object owner field name should be owner"
                );
                filed_value
            }
            _ => bail!("Object owner field should be address"),
        };
        let value = match fields.next().expect("Object should have value") {
            (field_name, AnnotatedMoveValue::Struct(filed_value)) => {
                debug_assert!(
                    field_name.as_str() == "value",
                    "Object value field name should be value"
                );
                filed_value
            }
            _ => bail!("Object value field should be struct"),
        };
        Ok(Self::new_annotated_object(object_id, owner, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_to_object_id() {
        let address = AccountAddress::random();
        let object_id = ObjectID::from(address);
        let address2 = AccountAddress::from(object_id);
        assert_eq!(address, address2);
    }

    #[test]
    fn test_object_id_from_str() {
        let address = AccountAddress::random();
        let object_id = ObjectID::from(address);
        let object_id2 = ObjectID::from_str(&object_id.to_string()).unwrap();
        assert_eq!(object_id, object_id2);
    }

    #[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize)]
    struct TestStruct {
        v: u8,
    }

    impl MoveStructType for TestStruct {
        const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
        const MODULE_NAME: &'static IdentStr = ident_str!("test");
        const STRUCT_NAME: &'static IdentStr = ident_str!("TestStruct");
    }

    impl MoveStructState for TestStruct {
        fn struct_layout() -> MoveStructLayout {
            MoveStructLayout::new(vec![MoveTypeLayout::U8])
        }
    }

    #[test]
    fn test_object_serialize() {
        //let struct_type = TestStruct::struct_tag();
        let object_value = TestStruct { v: 1 };
        let object_id = ObjectID::new(crate::h256::H256::random().into());
        let object = Object::new(object_id, AccountAddress::random(), object_value);

        let raw_object: RawObject = object.to_raw();

        let object2 = bcs::from_bytes::<Object<TestStruct>>(&raw_object.to_bytes()).unwrap();
        assert_eq!(object, object2);
    }

    #[test]
    fn test_named_table_id() {
        //ensure the table id is same as the table id in move
        let addr = AccountAddress::from_hex_literal(
            "0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647",
        )
        .unwrap();
        let resource_table_id = NamedTableID::Resource(addr).to_object_id();
        let module_table_id = NamedTableID::Module(addr).to_object_id();
        print!("{:?} {:?}", resource_table_id, module_table_id)
    }

    fn test_object_id_roundtrip(object_id: ObjectID) {
        let object_id_str = object_id.to_string();
        //ensure the ObjectID to string is hex with 0x prefix
        //and is full 32 bytes output
        assert!(object_id_str.starts_with("0x"));
        assert_eq!(object_id_str.len(), 66);
        let object_id_from_str = ObjectID::from_str(&object_id_str).unwrap();
        assert_eq!(object_id, object_id_from_str);

        let json_str = serde_json::to_string(&object_id).unwrap();
        assert_eq!(format!("\"{}\"", object_id_str), json_str);
        let object_id_from_json: ObjectID = serde_json::from_str(&json_str).unwrap();
        assert_eq!(object_id, object_id_from_json);

        let bytes = bcs::to_bytes(&object_id).unwrap();
        assert!(bytes.len() == 32);
        let object_id_from_bytes = bcs::from_bytes(&bytes).unwrap();
        assert_eq!(object_id, object_id_from_bytes);
    }

    #[test]
    fn test_object_id() {
        test_object_id_roundtrip(ObjectID::ZERO);
        test_object_id_roundtrip(ObjectID::ONE);
        test_object_id_roundtrip(ObjectID::new(crate::h256::H256::random().into()));
    }
}
