// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// The Move Object is from Sui Move, and we try to mix the Global storage module and Object model in MoveOS.
use anyhow::{bail, Result};
use move_core_types::{
    account_address::AccountAddress, language_storage::StructTag, move_resource::MoveResource,
};
use move_table_extension::TableHandle;
use serde::{Deserialize, Serialize};
use smt::HashValue;

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
pub struct ObjectID(HashValue);

impl ObjectID {
    const LENGTH: usize = HashValue::LENGTH;

    /// Creates a new ObjectID
    pub fn new(obj_id: [u8; Self::LENGTH]) -> Self {
        Self(HashValue::new(obj_id))
    }

    /// Create an ObjectID from transaction hash digest and `creation_num`.
    /// Caller is responsible for ensuring that hash is unique and
    /// `creation_num` is fresh
    pub fn derive_id(mut tx_hash: Vec<u8>, creation_num: u64) -> Self {
        tx_hash.extend(creation_num.to_le_bytes());
        ObjectID(HashValue::sha3_256_of(&tx_hash))
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, ObjectIDParseError> {
        <[u8; Self::LENGTH]>::try_from(bytes.as_ref())
            .map_err(|_| ObjectIDParseError::TryFromSliceError)
            .map(ObjectID::from)
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
        ObjectID(HashValue::new(address.into()))
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
                ObjectID(HashValue::sha3_256_of(&bytes))
            }
            NamedTableID::Module(address) => {
                let mut bytes = address.to_vec();
                bytes.push(1);
                ObjectID(HashValue::sha3_256_of(&bytes))
            }
        }
    }
}

impl From<TableHandle> for ObjectID {
    fn from(table_handle: TableHandle) -> Self {
        ObjectID(HashValue::new(table_handle.0.into()))
    }
}

pub type SequenceNumber = u64;

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct Object {
    pub data: ObjectData,
    /// The owner that unlocks this object
    pub owner: Owner,
}

impl Object {
    pub fn new_table_object(object: TableObject) -> Self {
        Self {
            data: ObjectData::TableObject(object),
            //TODO: set the owner
            owner: Owner::Immutable,
        }
    }

    pub fn new_move_object(object: MoveObject) -> Self {
        Self {
            data: ObjectData::MoveObject(object),
            //TODO: set the owner
            owner: Owner::Immutable,
        }
    }
}

impl TryInto<TableObject> for ObjectData {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<TableObject, Self::Error> {
        match self {
            ObjectData::TableObject(object) => Ok(object),
            ObjectData::MoveObject(_) => {
                bail!("expect table object, but get move object")
            }
        }
    }
}

impl TryInto<MoveObject> for ObjectData {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<MoveObject, Self::Error> {
        match self {
            ObjectData::TableObject(_) => bail!("expect move object, but get table object"),
            ObjectData::MoveObject(object) => Ok(object),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub enum ObjectData {
    TableObject(TableObject),
    MoveObject(MoveObject),
}

impl ObjectData {
    pub fn as_table_object(&self) -> Result<&TableObject> {
        match self {
            ObjectData::TableObject(object) => Ok(object),
            ObjectData::MoveObject(_) => bail!("object is not a table object"),
        }
    }

    pub fn as_table_object_mut(&mut self) -> Result<&mut TableObject> {
        match self {
            ObjectData::TableObject(object) => Ok(object),
            ObjectData::MoveObject(_) => bail!("object is not a table object"),
        }
    }

    pub fn as_move_object(&self) -> Result<&MoveObject> {
        match self {
            ObjectData::TableObject(_) => bail!("object is not a move object"),
            ObjectData::MoveObject(object) => Ok(object),
        }
    }

    pub fn as_move_object_mut(&mut self) -> Result<&mut MoveObject> {
        match self {
            ObjectData::TableObject(_) => bail!("object is not a move object"),
            ObjectData::MoveObject(object) => Ok(object),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct MoveObject {
    pub type_: StructTag,
    version: SequenceNumber,
    #[serde(with = "serde_bytes")]
    contents: Vec<u8>,
}

impl MoveObject {
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
pub struct TableObject {
    pub state_root: HashValue,
    pub version: SequenceNumber,
}

impl TableObject {
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
