// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
use super::{account_storage::AccountStorage, raw_table::TableInfo};
use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    h256,
    state::{MoveState, MoveStructState, MoveStructType, State},
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

pub const MODULE_NAME: &IdentStr = ident_str!("object");

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
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
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
        //Support singleton struct_type style object id
        if s.contains("::") {
            let struct_tag = StructTag::from_str(s).map_err(|_e| {
                anyhow::anyhow!("Singleton ObjectID must be a valid struct tag:{}", s)
            })?;
            Ok(singleton_object_id(&struct_tag))
        } else {
            let address = AccountAddress::from_hex_literal(s)
                .map_err(|_e| anyhow::anyhow!("Invalid ObjectID:{}|", s))?;
            Ok(ObjectID::from(address))
        }
    }
}

pub type TableObject = ObjectEntity<TableInfo>;
pub type AccountStorageObject = ObjectEntity<AccountStorage>;

/// The Entity of the Object<T>
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct ObjectEntity<T> {
    pub id: ObjectID,
    pub owner: AccountAddress,
    pub flag: u8,
    pub value: T,
}

impl<T> ObjectEntity<T> {
    const SHARED_OBJECT_FLAG_MASK: u8 = 1;
    const FROZEN_OBJECT_FLAG_MASK: u8 = 1 << 1;
    pub fn new(id: ObjectID, owner: AccountAddress, flag: u8, value: T) -> ObjectEntity<T> {
        Self {
            id,
            owner,
            flag,
            value,
        }
    }

    pub fn is_shared(&self) -> bool {
        self.flag & Self::SHARED_OBJECT_FLAG_MASK == Self::SHARED_OBJECT_FLAG_MASK
    }

    pub fn is_frozen(&self) -> bool {
        self.flag & Self::FROZEN_OBJECT_FLAG_MASK == Self::FROZEN_OBJECT_FLAG_MASK
    }
}

impl<T> ObjectEntity<T>
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
            flag: self.flag,
            value: RawData {
                struct_tag: T::struct_tag(),
                value: bcs::to_bytes(&self.value).expect("MoveState to bcs should success"),
            },
        }
    }
}

impl<T> From<ObjectEntity<T>> for RawObject
where
    T: MoveStructState,
{
    fn from(object: ObjectEntity<T>) -> Self {
        object.to_raw()
    }
}

impl ObjectEntity<TableInfo> {
    pub fn new_table_object(id: ObjectID, value: TableInfo) -> TableObject {
        Self {
            id,
            owner: AccountAddress::ZERO,
            flag: 0u8,
            value,
        }
    }
}

impl ObjectEntity<AccountStorage> {
    pub fn new_account_storage_object(account: AccountAddress) -> AccountStorageObject {
        Self {
            id: ObjectID::from(account),
            owner: account,
            flag: 0u8,
            value: AccountStorage::new(account),
        }
    }
}

impl<T> MoveStructType for ObjectEntity<T>
where
    T: MoveStructType,
{
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("ObjectEntity");

    fn type_params() -> Vec<TypeTag> {
        vec![TypeTag::Struct(Box::new(T::struct_tag()))]
    }
}

impl<T> MoveStructState for ObjectEntity<T>
where
    T: MoveStructState,
{
    /// Return the layout of the Object in Move
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::Struct(ObjectID::struct_layout()),
            MoveTypeLayout::Address,
            MoveTypeLayout::U8,
            MoveTypeLayout::Struct(T::struct_layout()),
        ])
    }
}

pub type RawObject = ObjectEntity<RawData>;

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
        let owner: AccountAddress =
            bcs::from_bytes(&bytes[ObjectID::LENGTH..ObjectID::LENGTH + AccountAddress::LENGTH])?;
        let flag = bytes[ObjectID::LENGTH + AccountAddress::LENGTH
            ..ObjectID::LENGTH + AccountAddress::LENGTH + 1][0];
        let value = bytes[ObjectID::LENGTH + AccountAddress::LENGTH + 1..].to_vec();
        Ok(RawObject {
            id,
            owner,
            flag,
            value: RawData { struct_tag, value },
        })
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(bcs::to_bytes(&self.id).unwrap());
        bytes.extend(bcs::to_bytes(&self.owner).unwrap());
        bytes.push(self.flag);
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

pub type AnnotatedObject = ObjectEntity<AnnotatedMoveStruct>;

impl AnnotatedObject {
    pub fn new_annotated_object(
        id: ObjectID,
        owner: AccountAddress,
        flag: u8,
        value: AnnotatedMoveStruct,
    ) -> Self {
        Self::new(id, owner, flag, value)
    }

    /// Create a new AnnotatedObject from a AnnotatedMoveStruct
    /// The MoveStruct is ObjectEntity<T> in Move, not the T
    pub fn new_from_annotated_struct(object_struct: AnnotatedMoveStruct) -> Result<Self> {
        let mut fields = object_struct.value.into_iter();
        let object_id = ObjectID::try_from(fields.next().expect("ObjectEntity should have id").1)?;
        let owner = match fields.next().expect("ObjectEntity should have owner") {
            (field_name, AnnotatedMoveValue::Address(field_value)) => {
                debug_assert!(
                    field_name.as_str() == "owner",
                    "ObjectEntity owner field name should be owner"
                );
                field_value
            }
            _ => bail!("ObjectEntity owner field should be address"),
        };
        let flag = match fields.next().expect("ObjectEntity should have flag") {
            (field_name, AnnotatedMoveValue::U8(field_value)) => {
                debug_assert!(
                    field_name.as_str() == "flag",
                    "ObjectEntity flag field name should be flag"
                );
                field_value
            }
            _ => bail!("ObjectEntity flag field should be u8"),
        };
        let value = match fields.next().expect("ObjectEntity should have value") {
            (field_name, AnnotatedMoveValue::Struct(field_value)) => {
                debug_assert!(
                    field_name.as_str() == "value",
                    "ObjectEntity value field name should be value"
                );
                field_value
            }
            _ => bail!("ObjectEntity value field should be struct"),
        };
        Ok(Self::new_annotated_object(object_id, owner, flag, value))
    }
}

/// In Move, Object<T> is like a pointer to ObjectEntity<T>
#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Object<T> {
    pub id: ObjectID,
    pub ty: std::marker::PhantomData<T>,
}

impl<T> MoveStructType for Object<T>
where
    T: MoveStructType,
{
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Object");

    fn type_params() -> Vec<TypeTag> {
        vec![T::type_tag()]
    }
}

impl<T> MoveStructState for Object<T>
where
    T: MoveStructType,
{
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![ObjectID::type_layout()])
    }
}

pub fn singleton_object_id(struct_tag: &StructTag) -> ObjectID {
    let struct_tag_hash = h256::sha3_256_of(struct_tag.to_canonical_string().as_bytes());
    AccountAddress::new(struct_tag_hash.0).into()
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
        let object = ObjectEntity::new(object_id, AccountAddress::random(), 0u8, object_value);

        let raw_object: RawObject = object.to_raw();

        let object2 = bcs::from_bytes::<ObjectEntity<TestStruct>>(&raw_object.to_bytes()).unwrap();
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

    #[test]
    fn test_singleton_object_id() {
        let struct_tag = StructTag {
            address: AccountAddress::from_str("0x3").unwrap(),
            module: ident_str!("timestamp").to_owned(),
            name: ident_str!("Timestamp").to_owned(),
            type_params: vec![],
        };
        let timestamp_object_id = singleton_object_id(&struct_tag);
        let object_id = ObjectID::from_str(
            "0x711ab0301fd517b135b88f57e84f254c94758998a602596be8ae7ba56a0d14b3",
        )
        .unwrap();
        let timestamp_object_id2 = ObjectID::from_str("0x3::timestamp::Timestamp").unwrap();
        assert_eq!(timestamp_object_id, object_id,);
        assert_eq!(timestamp_object_id2, object_id,);
    }
}
