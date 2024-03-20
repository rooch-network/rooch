// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::table::TablePlaceholder;
use crate::h256;
use crate::moveos_std::account::Account;
use crate::moveos_std::move_module::ModuleStore;
use crate::state::KeyState;
use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveState, MoveStructState, MoveStructType, State},
};
use anyhow::{anyhow, bail, ensure, Result};
use fastcrypto::encoding::Hex;
use move_core_types::language_storage::ModuleId;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::{StructTag, TypeTag},
    value::{MoveStructLayout, MoveTypeLayout},
};
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use move_vm_types::values::Struct;
use move_vm_types::values::VMValueCast;
use move_vm_types::values::Value;
use once_cell::sync::Lazy;
use primitive_types::H256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smt::SPARSE_MERKLE_PLACEHOLDER_HASH;
use std::str::FromStr;

pub const MODULE_NAME: &IdentStr = ident_str!("object");
pub static MODULE_ID: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(MOVEOS_STD_ADDRESS, MODULE_NAME.to_owned()));
pub const OBJECT_ENTITY_STRUCT_NAME: &IdentStr = ident_str!("ObjectEntity");

// New table's state_root should be the place holder hash.
pub static GENESIS_STATE_ROOT: Lazy<H256> = Lazy::new(|| *SPARSE_MERKLE_PLACEHOLDER_HASH);

#[derive(Eq, PartialEq, Clone, PartialOrd, Ord, Hash, JsonSchema)]
pub struct ObjectID(#[schemars(with = "Hex")] Vec<AccountAddress>);

impl ObjectID {
    /// Creates a new ObjectID
    pub fn new(obj_id: [u8; AccountAddress::LENGTH]) -> Self {
        Self(vec![AccountAddress::new(obj_id)])
    }

    pub fn root() -> Self {
        Self(vec![])
    }

    pub fn random() -> Self {
        Self::new(h256::H256::random().into())
    }

    pub fn zero() -> Self {
        Self::new([0u8; AccountAddress::LENGTH])
    }

    /// Create an ObjectID from transaction hash digest and `creation_num`.
    /// Caller is responsible for ensuring that hash is unique and
    /// `creation_num` is fresh
    pub fn derive_id(tx_hash: Vec<u8>, creation_num: u64) -> Self {
        let mut buffer = tx_hash;
        buffer.extend(creation_num.to_le_bytes());
        Self::new(h256::sha3_256_of(&buffer).into())
    }

    pub fn to_key(&self) -> KeyState {
        let key_type = TypeTag::Struct(Box::new(Self::struct_tag()));
        //We should use the bcs::to_bytes(ObjectID) as the key
        KeyState::new(self.to_bytes(), key_type)
    }

    pub fn to_hex(&self) -> String {
        let bytes: Vec<u8> = self.0.iter().flat_map(|addr| addr.to_vec()).collect();
        hex::encode(bytes)
    }

    pub fn from_hex(hex: &str) -> Result<Self> {
        let bytes = hex::decode(hex)?;
        bytes
            .chunks_exact(AccountAddress::LENGTH)
            .map(|chunk| {
                let mut addr = [0u8; AccountAddress::LENGTH];
                addr.copy_from_slice(chunk);
                Ok(AccountAddress::new(addr))
            })
            .collect::<Result<Vec<AccountAddress>>>()
            .map(Self)
    }

    pub fn from_hex_literal(literal: &str) -> Result<Self> {
        let literal = literal.strip_prefix("0x").unwrap_or(literal);
        let hex_len = literal.len();
        // If the string is too short, pad it
        if hex_len < AccountAddress::LENGTH * 2 {
            let mut hex_str = String::with_capacity(AccountAddress::LENGTH * 2);
            for _ in 0..AccountAddress::LENGTH * 2 - hex_len {
                hex_str.push('0');
            }
            hex_str.push_str(literal);
            Self::from_hex(hex_str.as_str())
        } else {
            Self::from_hex(literal)
        }
    }
}

impl std::fmt::Display for ObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", self.to_hex())
    }
}

impl std::fmt::Debug for ObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", self.to_hex())
    }
}

impl MoveStructType for ObjectID {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("ObjectID");
}

impl MoveStructState for ObjectID {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Vector(Box::new(
            MoveTypeLayout::Address,
        ))])
    }

    fn from_runtime_value_struct(value: Struct) -> Result<Self>
    where
        Self: Sized,
    {
        let mut fields = value.unpack()?;
        let vector = fields
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid ObjectID"))?;
        let path = vector.cast()?;
        Ok(ObjectID(path))
    }

    fn to_runtime_value_struct(&self) -> Struct {
        Struct::pack(vec![Value::vector_address(self.0.clone())])
    }
}

impl Serialize for ObjectID {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(self.to_string().as_str())
        } else {
            // See comment in deserialize.
            serializer.serialize_newtype_struct("ObjectID", &self.0)
        }
    }
}

impl<'de> Deserialize<'de> for ObjectID {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Ok(ObjectID::from_hex_literal(s.as_str()).map_err(serde::de::Error::custom)?)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "ObjectID")]
            struct Value(Vec<AccountAddress>);

            let value = Value::deserialize(deserializer)?;
            Ok(Self(value.0))
        }
    }
}

/// Try to convert moveos_std::object::ObjectID' MoveValue to ObjectID
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
        debug_assert!(field_name.as_str() == "path");
        let path = match field_value {
            AnnotatedMoveValue::Vector(t, vector) => {
                debug_assert!(t == TypeTag::Address);
                vector
                    .into_iter()
                    .map(|annotated_move_value| match annotated_move_value {
                        AnnotatedMoveValue::Address(addr) => Ok(addr),
                        _ => Err(anyhow::anyhow!("Invalid ObjectID")),
                    })
                    .collect::<Result<Vec<AccountAddress>>>()?
            }
            _ => return Err(anyhow::anyhow!("Invalid ObjectID")),
        };
        Ok(ObjectID(path))
    }
}

impl From<AccountAddress> for ObjectID {
    fn from(address: AccountAddress) -> Self {
        ObjectID(vec![address])
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
            ObjectID::from_hex_literal(s)
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

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Default)]
pub struct Root {
    // Move VM will auto add a bool field to the empty struct
    // So we manually add a bool field to the struct
    _placeholder: bool,
}

impl MoveStructType for Root {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Root");

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![],
        }
    }
}

impl MoveStructState for Root {
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![MoveTypeLayout::Bool])
    }
}

pub type TableObject = ObjectEntity<TablePlaceholder>;
pub type AccountObject = ObjectEntity<Account>;
pub type ModuleStoreObject = ObjectEntity<ModuleStore>;

/// The Entity of the Object<T>.
/// The value must be the last field
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct ObjectEntity<T> {
    pub id: ObjectID,
    pub owner: AccountAddress,
    pub flag: u8,
    /// The state tree root of the object dynamic fields
    pub state_root: AccountAddress,
    pub size: u64,
    pub value: T,
}

impl ObjectEntity<Root> {
    pub fn genesis_root_object() -> RootObjectEntity {
        Self::root_object(*GENESIS_STATE_ROOT, 0)
    }

    pub fn root_object(state_root: H256, size: u64) -> RootObjectEntity {
        Self {
            id: ObjectID::root(),
            owner: MOVEOS_STD_ADDRESS,
            flag: 0u8,
            state_root: AccountAddress::new(state_root.into()),
            size,
            value: Root {
                _placeholder: false,
            },
        }
    }
}

impl<T> ObjectEntity<T> {
    const SHARED_OBJECT_FLAG_MASK: u8 = 1;
    const FROZEN_OBJECT_FLAG_MASK: u8 = 1 << 1;

    pub fn new(
        id: ObjectID,
        owner: AccountAddress,
        flag: u8,
        state_root: H256,
        size: u64,
        value: T,
    ) -> ObjectEntity<T> {
        Self {
            id,
            owner,
            flag,
            state_root: AccountAddress::new(state_root.into()),
            size,
            value,
        }
    }

    pub fn state_root(&self) -> H256 {
        self.state_root.into_bytes().into()
    }

    pub fn update_state_root(&mut self, new_state_root: H256) {
        self.state_root = AccountAddress::new(new_state_root.into());
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
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bcs::to_bytes(self)
            .map_err(|e| anyhow::anyhow!("Serialize the ObjectEntity error: {:?}", e))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        bcs::from_bytes(bytes)
            .map_err(|e| anyhow::anyhow!("Deserialize the ObjectEntity error: {:?}", e))
    }

    pub fn to_raw(&self) -> RawObject {
        RawObject {
            id: self.id.clone(),
            owner: self.owner,
            flag: self.flag,
            value: RawData {
                struct_tag: T::struct_tag(),
                value: bcs::to_bytes(&self.value).expect("MoveState to bcs should success"),
            },
            state_root: self.state_root,
            size: self.size,
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

impl ObjectEntity<TablePlaceholder> {
    pub fn new_table_object(id: ObjectID, state_root: H256, size: u64) -> TableObject {
        Self::new(
            id,
            AccountAddress::ZERO,
            0u8,
            state_root,
            size,
            TablePlaceholder::default(),
        )
    }

    pub fn get_table_object_struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![TablePlaceholder::struct_tag().into()],
        }
    }
}

impl ObjectEntity<Account> {
    pub fn new_account_object(account: AccountAddress) -> AccountObject {
        Self::new(
            Account::account_object_id(account),
            account,
            0u8,
            *GENESIS_STATE_ROOT,
            0,
            Account::default(),
        )
    }
}

impl ObjectEntity<ModuleStore> {
    pub fn new_module_store() -> ModuleStoreObject {
        Self::new(
            ModuleStore::module_store_id(),
            MOVEOS_STD_ADDRESS,
            0u8,
            *GENESIS_STATE_ROOT,
            0,
            ModuleStore::default(),
        )
    }
}

impl<T> MoveStructType for ObjectEntity<T>
where
    T: MoveStructType,
{
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = OBJECT_ENTITY_STRUCT_NAME;

    fn type_params() -> Vec<TypeTag> {
        vec![TypeTag::Struct(Box::new(T::struct_tag()))]
    }

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![T::struct_tag().into()],
        }
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
            MoveTypeLayout::Address,
            MoveTypeLayout::U64,
            MoveTypeLayout::Struct(T::struct_layout()),
        ])
    }
}

//TODO rename to RawObjectEntity
pub type RawObject = ObjectEntity<RawData>;
pub type RootObjectEntity = ObjectEntity<Root>;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct RawData {
    pub struct_tag: StructTag,
    pub value: Vec<u8>,
}

impl RawObject {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = OBJECT_ENTITY_STRUCT_NAME;

    //This function is from bcs module,
    //find a better way to parse the vec.
    fn parse_length(bytes: &[u8]) -> Result<(usize, usize)> {
        let mut value: u64 = 0;
        let mut iter = bytes.iter();
        let mut used_bytes: usize = 0;
        for shift in (0..32).step_by(7) {
            let byte = *iter
                .next()
                .ok_or_else(|| anyhow!("Invalid bytes, NonCanonicalUleb128Encoding"))?;
            used_bytes += 1;
            let digit = byte & 0x7f;
            value |= u64::from(digit) << shift;
            // If the highest bit of `byte` is 0, return the final value.
            if digit == byte {
                if shift > 0 && digit == 0 {
                    // We only accept canonical ULEB128 encodings, therefore the
                    // heaviest (and last) base-128 digit must be non-zero.
                    bail!("Invalid bytes, NonCanonicalUleb128Encoding");
                }
                // Decoded integer must not overflow.
                return Ok((
                    used_bytes,
                    u32::try_from(value).map_err(|_| {
                        anyhow!("Invalid bytes, IntegerOverflowDuringUleb128Decoding")
                    })? as usize,
                ));
            }
        }
        // Decoded integer must not overflow.
        bail!("Invalid bytes, IntegerOverflowDuringUleb128Decoding")
    }

    pub fn from_bytes(bytes: &[u8], struct_tag: StructTag) -> Result<Self> {
        let (path_len_bytes, path_len) = Self::parse_length(bytes)?;
        let object_id_len = path_len_bytes + path_len * AccountAddress::LENGTH;

        ensure!(
            bytes.len() > object_id_len + AccountAddress::LENGTH + 1 + AccountAddress::LENGTH + 8,
            "Invalid bytes length"
        );

        let id: ObjectID = bcs::from_bytes(&bytes[..object_id_len])?;
        let owner: AccountAddress =
            bcs::from_bytes(&bytes[object_id_len..object_id_len + AccountAddress::LENGTH])?;
        let flag = bytes
            [object_id_len + AccountAddress::LENGTH..object_id_len + AccountAddress::LENGTH + 1][0];
        let state_root: AccountAddress = bcs::from_bytes(
            &bytes[object_id_len + AccountAddress::LENGTH + 1
                ..object_id_len + AccountAddress::LENGTH + 1 + AccountAddress::LENGTH],
        )?;
        let size: u64 = bcs::from_bytes(
            &bytes[object_id_len + AccountAddress::LENGTH + 1 + AccountAddress::LENGTH
                ..object_id_len + AccountAddress::LENGTH + 1 + AccountAddress::LENGTH + 8],
        )?;
        let value = bytes
            [object_id_len + AccountAddress::LENGTH + 1 + AccountAddress::LENGTH + 8..]
            .to_vec();
        Ok(RawObject {
            id,
            owner,
            flag,
            value: RawData { struct_tag, value },
            state_root,
            size,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(bcs::to_bytes(&self.id).unwrap());
        bytes.extend(bcs::to_bytes(&self.owner).unwrap());
        bytes.push(self.flag);
        bytes.extend(bcs::to_bytes(&self.state_root).unwrap());
        bytes.extend(bcs::to_bytes(&self.size).unwrap());
        bytes.extend_from_slice(&self.value.value);
        bytes
    }

    fn struct_tag(&self) -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![self.value.struct_tag.clone().into()],
        }
    }

    // The output must consistent with ObjectEntity<T> into state result
    pub fn into_state(&self) -> State {
        let value = self.to_bytes();
        let value_type = TypeTag::Struct(Box::new(self.struct_tag()));
        State::new(value, value_type)
    }

    pub fn into_object<T: MoveStructState>(self) -> Result<ObjectEntity<T>> {
        let struct_tag = T::struct_tag();
        ensure!(
            self.value.struct_tag == struct_tag,
            "RawObjectEntity value type should be {}",
            struct_tag
        );
        let value = bcs::from_bytes(&self.value.value)?;
        Ok(ObjectEntity {
            id: self.id,
            owner: self.owner,
            flag: self.flag,
            state_root: self.state_root,
            size: self.size,
            value,
        })
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
        state_root: AccountAddress,
        size: u64,
        value: AnnotatedMoveStruct,
    ) -> Self {
        Self {
            id,
            owner,
            flag,
            state_root,
            size,
            value,
        }
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
        let state_root = match fields.next().expect("ObjectEntity should have state_root") {
            (field_name, AnnotatedMoveValue::Address(field_value)) => {
                debug_assert!(
                    field_name.as_str() == "state_root",
                    "ObjectEntity state_root field name should be state_root"
                );
                field_value
            }
            _ => bail!("ObjectEntity state_root field should be address"),
        };
        let size = match fields.next().expect("ObjectEntity should have size") {
            (field_name, AnnotatedMoveValue::U64(field_value)) => {
                debug_assert!(
                    field_name.as_str() == "size",
                    "ObjectEntity size field name should be size"
                );
                field_value
            }
            _ => bail!("ObjectEntity size field should be u64"),
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
        Ok(Self::new_annotated_object(
            object_id, owner, flag, state_root, size, value,
        ))
    }
}

/// In Move, Object<T> is like a pointer to ObjectEntity<T>
#[derive(Debug, Eq, PartialEq, Clone, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use move_vm_types::values::Value;

    use super::*;
    use anyhow::Ok;

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
    fn test_object_serialize() -> Result<()> {
        //let struct_type = TestStruct::struct_tag();
        let object_value = TestStruct { count: 1 };
        let object_id = ObjectID::random();
        let object = ObjectEntity::new(
            object_id,
            AccountAddress::random(),
            0u8,
            H256::random(),
            0,
            object_value,
        );

        let bytes = bcs::to_bytes(&object)?;

        let raw_object: RawObject = RawObject::from_bytes(&bytes, TestStruct::struct_tag())?;

        let object2 = bcs::from_bytes::<ObjectEntity<TestStruct>>(&raw_object.to_bytes()).unwrap();
        assert_eq!(object, object2);

        let runtime_value = Value::simple_deserialize(
            &raw_object.into_state().value,
            &ObjectEntity::<TestStruct>::type_layout(),
        )
        .unwrap();
        let object3 = ObjectEntity::<TestStruct>::from_runtime_value(runtime_value)?;
        assert_eq!(object, object3);
        Ok(())
    }

    #[test]
    fn test_root_object() {
        let root_object = RootObjectEntity::genesis_root_object();
        let raw_object: RawObject =
            RawObject::from_bytes(&root_object.to_bytes().unwrap(), Root::struct_tag()).unwrap();
        let state = raw_object.into_state();

        let object = raw_object.into_object::<Root>().unwrap();
        assert_eq!(root_object, object);
        let runtime_value =
            Value::simple_deserialize(&state.value, &RootObjectEntity::type_layout()).unwrap();
        let object2 = RootObjectEntity::from_runtime_value(runtime_value).unwrap();
        assert_eq!(root_object, object2);
    }

    #[test]
    fn test_genesis_state_root() {
        let genesis_state_root = *GENESIS_STATE_ROOT;
        //println!("genesis_state_root: {:?}", genesis_state_root);
        //ensure the genesis state root is not changed
        assert_eq!(
            genesis_state_root,
            H256::from_str("0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000")
                .unwrap()
        );
    }

    #[test]
    fn test_address_to_object_id() {
        let address = AccountAddress::random();
        let object_id = ObjectID::from(address);
        assert_eq!(address, object_id.0[0]);
    }

    #[test]
    fn test_object_id_from_str() {
        let address = AccountAddress::random();
        let object_id = ObjectID::from(address);
        let object_id2 = ObjectID::from_str(&object_id.to_string()).unwrap();
        assert_eq!(object_id, object_id2);
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

        let object_id_from_bytes = bcs::from_bytes(&bytes).unwrap();
        assert_eq!(object_id, object_id_from_bytes);
    }

    #[test]
    fn test_object_id() {
        test_object_id_roundtrip(ObjectID::zero());
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

    #[test]
    fn test_to_move_value() {
        let object_id = ObjectID::random();
        let move_value = object_id.to_move_value();
        let object_id2 = ObjectID::from_bytes(move_value.simple_serialize().unwrap()).unwrap();
        assert_eq!(object_id, object_id2);
    }

    #[test]
    fn test_root_object_id() {
        let root_object_id = ObjectID::root();
        let bytes = root_object_id.to_bytes();
        assert!(bytes == vec![0u8]);
    }
}
