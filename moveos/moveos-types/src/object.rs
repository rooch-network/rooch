// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
use crate::h256::H256;
/// The Move Object is from Sui Move, and we try to mix the Global storage model and Object model in MoveOS.
use anyhow::{ensure, Result};
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::TypeTag,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::str::FromStr;

use crate::h256;

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

#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjectID(AccountAddress);

impl ObjectID {
    const LENGTH: usize = h256::LENGTH;

    /// Creates a new ObjectID
    pub fn new(obj_id: [u8; Self::LENGTH]) -> Self {
        Self(AccountAddress::new(obj_id))
    }

    /// Create an ObjectID from transaction hash digest and `creation_num`.
    /// Caller is responsible for ensuring that hash is unique and
    /// `creation_num` is fresh
    pub fn derive_id(tx_hash: H256, creation_num: u64) -> Self {
        let mut buffer = tx_hash.0.to_vec();
        buffer.extend(creation_num.to_le_bytes());
        Self::new(h256::sha3_256_of(&buffer).into())
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, ObjectIDParseError> {
        <[u8; Self::LENGTH]>::try_from(bytes.as_ref())
            .map_err(|_| ObjectIDParseError::TryFromSliceError)
            .map(ObjectID::from)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl std::fmt::Display for ObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_hex_literal())
    }
}

#[derive(PartialEq, Eq, Clone, Debug, thiserror::Error)]
pub enum ObjectIDParseError {
    #[error("ObjectID hex literal must start with 0x")]
    HexLiteralPrefixMissing,

    #[error("ObjectID hex string should only contain 0-9, A-F, a-f")]
    InvalidHexCharacter,

    #[error("hex string must be even-numbered. Two chars maps to one byte.")]
    OddLength,

    #[error("ObjectID must be {} bytes long.", ObjectID::LENGTH)]
    InvalidLength,

    #[error("Could not convert from bytes slice")]
    TryFromSliceError,
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
            H256(named_object_id.account().into()),
            named_object_id.table_index(),
        )
    }
}

impl FromStr for ObjectID {
    type Err = ObjectIDParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let address = AccountAddress::from_hex_literal(s)
            .map_err(|_| ObjectIDParseError::InvalidHexCharacter)?;
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
    const MODULE_NAME: &'static IdentStr = ident_str!("account_storage");
    const STRUCT_NAME: &'static IdentStr = ident_str!("AccountStorage");

    fn type_params() -> Vec<TypeTag> {
        vec![]
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

impl MoveStructType for TableInfo {
    const MODULE_NAME: &'static IdentStr = ident_str!("raw_table");
    const STRUCT_NAME: &'static IdentStr = ident_str!("AccountStorage");

    fn type_params() -> Vec<TypeTag> {
        vec![]
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

impl<T> Object<T>
where
    T: MoveStructType,
{
    pub fn new(id: ObjectID, owner: AccountAddress, value: T) -> Object<T> {
        Self { id, owner, value }
    }
}

impl<T> Object<T>
where
    T: Serialize,
{
    pub fn to_bytes(&self) -> Vec<u8> {
        bcs::to_bytes(self).unwrap()
    }

    pub fn to_raw(&self) -> RawObject {
        RawObject {
            id: self.id,
            owner: self.owner,
            value: bcs::to_bytes(&self.value).unwrap(),
        }
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

impl<T> MoveResource for Object<T> where T: MoveStructType + DeserializeOwned {}

pub const OBJECT_MODULE_NAME: &IdentStr = ident_str!("object");
pub const OBJECT_STRUCT_NAME: &IdentStr = ident_str!("Object");

impl<T> MoveStructType for Object<T>
where
    T: MoveStructType,
{
    const MODULE_NAME: &'static IdentStr = OBJECT_MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = OBJECT_STRUCT_NAME;

    fn type_params() -> Vec<TypeTag> {
        vec![TypeTag::Struct(Box::new(T::struct_tag()))]
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct RawObject {
    pub id: ObjectID,
    pub owner: AccountAddress,
    pub value: Vec<u8>,
}

impl RawObject {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        ensure!(
            bytes.len() > ObjectID::LENGTH + AccountAddress::LENGTH,
            "Invalid bytes length"
        );

        let id: ObjectID = bcs::from_bytes(&bytes[..ObjectID::LENGTH])?;
        let owner: AccountAddress = bcs::from_bytes(
            &bytes[AccountAddress::LENGTH..ObjectID::LENGTH + AccountAddress::LENGTH],
        )?;
        let value = bytes[ObjectID::LENGTH + AccountAddress::LENGTH..].to_vec();
        Ok(RawObject { id, owner, value })
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(bcs::to_bytes(&self.id).unwrap());
        bytes.extend(bcs::to_bytes(&self.owner).unwrap());
        bytes.extend_from_slice(&self.value);
        bytes
    }
}

impl From<Object<Vec<u8>>> for RawObject {
    fn from(obj: Object<Vec<u8>>) -> Self {
        RawObject {
            id: obj.id,
            owner: obj.owner,
            value: obj.value,
        }
    }
}

impl From<RawObject> for Object<Vec<u8>> {
    fn from(val: RawObject) -> Self {
        Object {
            id: val.id,
            owner: val.owner,
            value: val.value,
        }
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
        const MODULE_NAME: &'static IdentStr = ident_str!("test");
        const STRUCT_NAME: &'static IdentStr = ident_str!("TestStruct");
    }

    impl MoveResource for TestStruct {}

    #[test]
    fn test_object_serialize() {
        //let struct_type = TestStruct::struct_tag();
        let object_value = TestStruct { v: 1 };
        let object_id = ObjectID::new(H256::random().into());
        let object = Object::new(object_id, AccountAddress::random(), object_value);

        let bytes = object.to_bytes();
        let raw_object: RawObject = RawObject::from_bytes(&bytes).unwrap();

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
}
