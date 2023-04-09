// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
/// The Move Object is from Sui Move, and we try to mix the Global storage model and Object model in MoveOS.
use anyhow::{bail, Result};
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::{StructTag, TypeTag},
    move_resource::{MoveResource, MoveStructType},
    value::{MoveStructLayout, MoveTypeLayout},
};
use move_table_extension::TableHandle;
use serde::{Deserialize, Serialize};
use smt::HashValue;
use std::str::FromStr;

/// Specific Table Object ID associated with an address
#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NamedTableID {
    Resource(AccountAddress),
    Module(AccountAddress),
}

impl NamedTableID {
    pub fn to_object_id(self) -> ObjectID {
        self.into()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjectID(AccountAddress);

impl ObjectID {
    const LENGTH: usize = HashValue::LENGTH;

    /// Creates a new ObjectID
    pub fn new(obj_id: [u8; Self::LENGTH]) -> Self {
        Self(AccountAddress::new(obj_id))
    }

    /// Create an ObjectID from transaction hash digest and `creation_num`.
    /// Caller is responsible for ensuring that hash is unique and
    /// `creation_num` is fresh
    pub fn derive_id(mut tx_hash: Vec<u8>, creation_num: u64) -> Self {
        tx_hash.extend(creation_num.to_le_bytes());
        Self::new(HashValue::sha3_256_of(&tx_hash).into())
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, ObjectIDParseError> {
        <[u8; Self::LENGTH]>::try_from(bytes.as_ref())
            .map_err(|_| ObjectIDParseError::TryFromSliceError)
            .map(ObjectID::from)
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
        match named_object_id {
            NamedTableID::Resource(address) => {
                let mut bytes = address.to_vec();
                bytes.push(0);
                ObjectID::new(HashValue::sha3_256_of(&bytes).into())
            }
            NamedTableID::Module(address) => {
                let mut bytes = address.to_vec();
                bytes.push(1);
                ObjectID::new(HashValue::sha3_256_of(&bytes).into())
            }
        }
    }
}

impl From<TableHandle> for ObjectID {
    fn from(table_handle: TableHandle) -> Self {
        ObjectID(table_handle.0)
    }
}

impl FromStr for ObjectID {
    type Err = ObjectIDParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let address = AccountAddress::from_hex_literal(s)
            .map_err(|_| ObjectIDParseError::InvalidHexCharacter)?;
        Ok(address.into())
    }
}

pub type SequenceNumber = u64;

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct Object {
    //TODO should contains self id.
    //pub id: ObjectID,
    pub data: ObjectData,
    /// The owner that unlocks this object
    pub owner: Owner,
}

impl Object {
    pub fn new_table_object(value: TableObjectData) -> Self {
        Self {
            data: ObjectData::Table(value),
            //TODO: set the owner
            owner: Owner::Immutable,
        }
    }

    pub fn new_move_object(value: MoveObjectData) -> Self {
        Self {
            data: ObjectData::Move(value),
            //TODO: set the owner
            owner: Owner::Immutable,
        }
    }

    /// Serialize the object into a byte array and as an argument to a Move function
    pub fn as_object_argument(&self, id: ObjectID) -> Result<Vec<u8>> {
        match &self.data {
            ObjectData::Table(_table_data) => {
                //TODO should support table as argument.
                bail!("table object is not supported")
            }
            ObjectData::Move(move_data) => {
                //TODO find a better way to concat id and value to MoveObject
                // Object in Move
                // struct Object<T>{
                //     id: ObjectID,
                //     value: T,
                // }
                let mut resource_bytes = vec![];
                resource_bytes.extend(id.0.to_vec());
                resource_bytes.extend_from_slice(move_data.contents.as_slice());

                Ok(resource_bytes)
            }
        }
    }
}

impl TryInto<TableObjectData> for ObjectData {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<TableObjectData, Self::Error> {
        match self {
            ObjectData::Table(object) => Ok(object),
            ObjectData::Move(_) => {
                bail!("expect table object, but get move object")
            }
        }
    }
}

impl TryInto<MoveObjectData> for ObjectData {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<MoveObjectData, Self::Error> {
        match self {
            ObjectData::Table(_) => bail!("expect move object, but get table object"),
            ObjectData::Move(object) => Ok(object),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub enum ObjectData {
    Table(TableObjectData),
    Move(MoveObjectData),
}

impl ObjectData {
    pub fn as_table_object(&self) -> Result<&TableObjectData> {
        match self {
            ObjectData::Table(object) => Ok(object),
            ObjectData::Move(_) => bail!("object is not a table object"),
        }
    }

    pub fn as_table_object_mut(&mut self) -> Result<&mut TableObjectData> {
        match self {
            ObjectData::Table(object) => Ok(object),
            ObjectData::Move(_) => bail!("object is not a table object"),
        }
    }

    pub fn as_move_object(&self) -> Result<&MoveObjectData> {
        match self {
            ObjectData::Table(_) => bail!("object is not a move object"),
            ObjectData::Move(object) => Ok(object),
        }
    }

    pub fn as_move_object_mut(&mut self) -> Result<&mut MoveObjectData> {
        match self {
            ObjectData::Table(_) => bail!("object is not a move object"),
            ObjectData::Move(object) => Ok(object),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct MoveObjectData {
    pub type_: StructTag,
    pub version: SequenceNumber,
    #[serde(with = "serde_bytes")]
    pub contents: Vec<u8>,
}

impl MoveObjectData {
    pub fn new(type_: StructTag, version: SequenceNumber, contents: Vec<u8>) -> Self {
        Self {
            type_,
            version,
            contents,
        }
    }
    pub fn decode<T: MoveResource>(&self) -> Result<T, anyhow::Error> {
        if T::struct_tag() != self.type_ {
            anyhow::bail!(
                "Type mismatch, expected: {:?}, got: {:?}",
                T::struct_tag(),
                self.type_
            );
        }
        Ok(bcs::from_bytes(&self.contents)?)
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct TableObjectData {
    pub state_root: HashValue,
    pub version: SequenceNumber,
}

impl TableObjectData {
    pub fn new(state_root: HashValue, version: SequenceNumber) -> Self {
        Self {
            state_root,
            version,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Deserialize, Serialize, Hash, Ord, PartialOrd)]
pub enum Owner {
    /// Object is exclusively owned by a single address, and is mutable.
    AddressOwner(AccountAddress),
    /// Object is exclusively owned by a single object, and is mutable.
    ObjectOwner(ObjectID),
    /// Object is shared, can be used by any address, and is mutable.
    Shared {
        /// The version at which the object became shared
        initial_shared_version: SequenceNumber,
    },
    /// Object is immutable, and hence ownership doesn't matter.
    Immutable,
}

pub const TABLE_MODULE_NAME: &IdentStr = ident_str!("table");
pub const TABLE_STRUCT_NAME: &IdentStr = ident_str!("Table");

///This struct is mapping to moveos_std::table::Table
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct MoveTable {
    pub handle: AccountAddress,
}

impl MoveStructType for MoveTable {
    const MODULE_NAME: &'static IdentStr = TABLE_MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = TABLE_STRUCT_NAME;
}

impl MoveResource for MoveTable {}

pub const OBJECT_MODULE_NAME: &IdentStr = ident_str!("object");
pub const OBJECT_STRUCT_NAME: &IdentStr = ident_str!("Object");

///This struct is mapping to moveos_std::object::Object and erasure the TypeTag
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct MoveObject {
    pub id: ObjectID,
    #[serde(with = "serde_bytes")]
    pub value: Vec<u8>,
}

impl MoveObject {
    pub fn object_layout(value_layout: MoveStructLayout) -> MoveTypeLayout {
        let fields = vec![
            MoveTypeLayout::Address,
            MoveTypeLayout::Struct(value_layout),
        ];
        MoveTypeLayout::Struct(MoveStructLayout::Runtime(fields))
    }
}

impl MoveStructType for MoveObject {
    const MODULE_NAME: &'static IdentStr = OBJECT_MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = OBJECT_STRUCT_NAME;

    fn type_params() -> Vec<TypeTag> {
        //TODO how to support Object<T>
        //vec![TypeTag::Struct(Box::new(T::struct_tag()))]
        vec![]
    }
}

impl MoveResource for MoveObject {}

//TODO find a better name
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct MoveObjectWithType<T> {
    pub id: ObjectID,
    pub value: T,
}

impl<T> MoveStructType for MoveObjectWithType<T>
where
    T: MoveStructType,
{
    const MODULE_NAME: &'static IdentStr = OBJECT_MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = OBJECT_STRUCT_NAME;

    fn type_params() -> Vec<TypeTag> {
        vec![TypeTag::Struct(Box::new(T::struct_tag()))]
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
        let struct_type = TestStruct::struct_tag();
        let object_value = TestStruct { v: 0 };
        let object_id = ObjectID::new(HashValue::random().into());
        let contents = bcs::to_bytes(&object_value).unwrap();
        let object = Object::new_move_object(MoveObjectData::new(struct_type, 0, contents.clone()));
        let _move_object = MoveObject {
            id: object_id,
            value: contents,
        };
        let move_object_with_type = MoveObjectWithType {
            id: object_id,
            value: object_value,
        };

        assert_eq!(
            object.as_object_argument(object_id).unwrap(),
            bcs::to_bytes(&move_object_with_type).unwrap()
        );
        //let move_object_bytes = bcs::to_bytes(&move_object).unwrap();

        //TODO fix this.
        //let move_object_with_type = bcs::from_bytes::<MoveObjectWithType<TestStruct>>(&move_object_bytes).unwrap();
        //assert_eq!(bcs::to_bytes(&move_object).unwrap(), bcs::to_bytes(&move_object_with_type).unwrap());
    }
}
