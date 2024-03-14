// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
use crate::state::{KeyState, MoveState};
use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    h256,
    state::{MoveStructState, MoveStructType, MoveType},
};
use anyhow::Result;
use fastcrypto::encoding::Hex;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::{StructTag, TypeTag},
    value::{MoveStructLayout, MoveTypeLayout},
};
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use move_vm_types::values::Struct;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::object::GLOBAL_OBJECT_STORAGE_HANDLE;

pub const MODULE_NAME: &IdentStr = ident_str!("object_id");

#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord, Hash, JsonSchema)]
pub struct ObjectID(#[schemars(with = "Hex")] AccountAddress);

impl ObjectID {
    pub const LENGTH: usize = h256::LENGTH;

    /// Creates a new ObjectID
    pub const fn new(obj_id: [u8; Self::LENGTH]) -> Self {
        Self(AccountAddress::new(obj_id))
    }

    pub fn root() -> Self {
        GLOBAL_OBJECT_STORAGE_HANDLE
    }

    pub fn random() -> Self {
        Self::new(h256::H256::random().into())
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

    pub fn from_key(key: KeyState) -> Result<Self> {
        if key.key_type != Self::type_tag() {
            return Err(anyhow::anyhow!("Invalid ObjectID type"));
        }
        Self::from_bytes(key.key)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn to_key(&self) -> KeyState {
        let key_type = TypeTag::Struct(Box::new(Self::struct_tag()));
        KeyState::new(self.to_bytes(), key_type)
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

    fn from_runtime_value_struct(value: Struct) -> Result<Self>
    where
        Self: Sized,
    {
        let mut fields = value.unpack()?;
        let address = AccountAddress::from_runtime_value(
            fields
                .next()
                .ok_or_else(|| anyhow::anyhow!("Invalid ObjectID"))?,
        )?;
        Ok(ObjectID(address))
    }

    fn to_runtime_value_struct(&self) -> Struct {
        Struct::pack(vec![self.0.to_runtime_value()])
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

impl FromStr for ObjectID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //Support named struct_type style object id
        if s.contains("::") {
            let struct_tag = StructTag::from_str(s)
                .map_err(|_e| anyhow::anyhow!("Named ObjectID must be a valid struct tag:{}", s))?;
            Ok(named_object_id(&struct_tag))
        } else {
            let address = AccountAddress::from_hex_literal(s)
                .map_err(|_e| anyhow::anyhow!("Invalid ObjectID:{}|", s))?;
            Ok(ObjectID::from(address))
        }
    }
}

pub fn named_object_id(struct_tag: &StructTag) -> ObjectID {
    let struct_tag_hash = h256::sha3_256_of(struct_tag.to_canonical_string().as_bytes());
    AccountAddress::new(struct_tag_hash.0).into()
}

pub fn account_named_object_id(account: AccountAddress, struct_tag: &StructTag) -> ObjectID {
    let mut buffer = account.to_vec();
    buffer.extend_from_slice(struct_tag.to_canonical_string().as_bytes());
    let struct_tag_hash = h256::sha3_256_of(&buffer);
    AccountAddress::new(struct_tag_hash.0).into()
}

pub fn custom_object_id<ID: Serialize>(id: ID, struct_tag: &StructTag) -> ObjectID {
    let mut buffer = bcs::to_bytes(&id).expect("ID to bcs should success");
    buffer.extend_from_slice(struct_tag.to_canonical_string().as_bytes());
    let struct_tag_hash = h256::sha3_256_of(&buffer);
    AccountAddress::new(struct_tag_hash.0).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moveos_std::account::Account;
    use crate::moveos_std::move_module::ModuleStore;

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
        count: u64,
    }

    impl MoveStructType for TestStruct {
        const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
        const MODULE_NAME: &'static IdentStr = ident_str!("object");
        const STRUCT_NAME: &'static IdentStr = ident_str!("TestStruct");
    }

    impl MoveStructState for TestStruct {
        fn struct_layout() -> MoveStructLayout {
            MoveStructLayout::new(vec![MoveTypeLayout::U64])
        }
    }

    #[test]
    fn test_resource_and_module_object_id() {
        //ensure the table id is same as the table id in move
        let addr = AccountAddress::from_hex_literal(
            "0xae43e34e51db9c833ab50dd9aa8b27106519e5bbfd533737306e7b69ef253647",
        )
        .unwrap();
        let account_object_id = Account::account_object_id(addr);
        let module_object_id = ModuleStore::module_store_id();
        print!("{:?} {:?}", account_object_id, module_object_id)
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
    fn test_named_object_id() {
        let struct_tag = StructTag {
            address: AccountAddress::from_str("0x3").unwrap(),
            module: ident_str!("timestamp").to_owned(),
            name: ident_str!("Timestamp").to_owned(),
            type_params: vec![],
        };
        let timestamp_object_id = named_object_id(&struct_tag);
        //The object id generated by crates/rooch-framework-tests/tests/cases/timestamp/timestamp_test.move
        let object_id = ObjectID::from_str(
            "0x711ab0301fd517b135b88f57e84f254c94758998a602596be8ae7ba56a0d14b3",
        )
        .unwrap();
        let timestamp_object_id2 = ObjectID::from_str("0x3::timestamp::Timestamp").unwrap();
        assert_eq!(timestamp_object_id, object_id,);
        assert_eq!(timestamp_object_id2, object_id,);
    }

    #[test]
    fn test_account_named_object_id() {
        let account = AccountAddress::from_str("0x42").unwrap();
        let struct_tag = StructTag {
            address: AccountAddress::from_str("0x3").unwrap(),
            module: ident_str!("coin_store").to_owned(),
            name: ident_str!("CoinStore").to_owned(),
            type_params: vec![TypeTag::Struct(Box::new(StructTag {
                address: AccountAddress::from_str("0x3").unwrap(),
                module: ident_str!("gas_coin").to_owned(),
                name: ident_str!("GasCoin").to_owned(),
                type_params: vec![],
            }))],
        };
        let coin_store_object_id = account_named_object_id(account, &struct_tag);
        //The object id generated by crates/rooch-framework-tests/tests/cases/coin_store/coin_store.move
        let object_id = ObjectID::from_str(
            "0xd073508b9582eff4e01078dc2e62489c15bbef91b6a2e568ac8fb33f0cf54daa",
        )
        .unwrap();
        assert_eq!(coin_store_object_id, object_id,);
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStructID {
        id: u64,
    }

    #[test]
    fn test_custom_object_id() {
        let id = TestStructID { id: 1 };
        let custom_object_id = custom_object_id(id, &TestStruct::struct_tag());
        //println!("custom_object_id: {:?}", custom_object_id);
        //Ensure the generated object id is same as the object id in object.move
        assert_eq!(
            custom_object_id,
            ObjectID::from_str(
                "0xaa825038ae811f5c94d20175699d808eae4c624fa85c81faad45de1145284e06"
            )
            .unwrap()
        );
    }

    #[test]
    fn test_from_runtime_value() {
        let object_id = ObjectID::random();
        let runtime_value = object_id.to_runtime_value();
        let object_id2 = ObjectID::from_runtime_value(runtime_value).unwrap();
        assert_eq!(object_id, object_id2);
    }
}
