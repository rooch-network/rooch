// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::table::TablePlaceholder;
use crate::h256;
use crate::moveos_std::account::Account;
use crate::moveos_std::module_store::{ModuleStore, Package};
use crate::moveos_std::timestamp::Timestamp;
use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{
        FieldKey, MoveState, MoveStructState, MoveStructType, MoveType, ObjectState,
        PlaceholderStruct,
    },
};
use anyhow::{ensure, Result};
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
use std::fmt;
use std::str::FromStr;

pub use dynamic_field::{
    construct_dynamic_field_struct_tag, is_dynamic_field_type, is_field_struct_tag, DynamicField,
    RawField, DYNAMIC_FIELD_STRUCT_NAME,
};

mod dynamic_field;

pub const MODULE_NAME: &IdentStr = ident_str!("object");
pub static MODULE_ID: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(MOVEOS_STD_ADDRESS, MODULE_NAME.to_owned()));
pub const OBJECT_ENTITY_STRUCT_NAME: &IdentStr = ident_str!("ObjectEntity");
pub const OBJECT_STRUCT_NAME: &IdentStr = ident_str!("Object");

pub const SYSTEM_OWNER_ADDRESS: AccountAddress = AccountAddress::ZERO;

pub const SHARED_OBJECT_FLAG_MASK: u8 = 1;
pub const FROZEN_OBJECT_FLAG_MASK: u8 = 1 << 1;

// New table's state_root should be the place holder hash.
pub static GENESIS_STATE_ROOT: Lazy<H256> = Lazy::new(|| *SPARSE_MERKLE_PLACEHOLDER_HASH);

/// The genesis state root in address format
pub static GENESIS_STATE_ROOT_ADDRESS: Lazy<AccountAddress> =
    Lazy::new(|| AccountAddress::new(GENESIS_STATE_ROOT.0));

pub fn human_readable_flag(flag: u8) -> String {
    if flag == 0 {
        return "UserOwned".to_string();
    };

    let mut status = vec![];
    if flag & SHARED_OBJECT_FLAG_MASK == SHARED_OBJECT_FLAG_MASK {
        status.push("Shared".to_string());
    }
    if flag & FROZEN_OBJECT_FLAG_MASK == FROZEN_OBJECT_FLAG_MASK {
        status.push("Frozen".to_string());
    }

    status.join(",")
}

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

    pub fn child_id(&self, child_key: FieldKey) -> Self {
        let mut path = self.0.clone();
        path.push(child_key.into());
        Self(path)
    }

    /// Create an ObjectID from transaction hash digest and `creation_num`.
    /// Caller is responsible for ensuring that hash is unique and
    /// `creation_num` is fresh
    pub fn derive_id(tx_hash: Vec<u8>, creation_num: u64) -> Self {
        let mut buffer = tx_hash;
        buffer.extend(creation_num.to_le_bytes());
        Self::new(h256::sha3_256_of(&buffer).into())
    }

    /// Get the parent ObjectID of the current ObjectID
    pub fn parent(&self) -> Option<Self> {
        if self.0.is_empty() {
            None
        } else {
            let mut parent = self.0.clone();
            parent.pop();
            Some(Self(parent))
        }
    }

    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    pub fn has_parent(&self) -> bool {
        !self.is_root()
    }

    pub fn has_child(&self) -> bool {
        self.is_root() || self.parent().is_some()
    }

    pub fn is_child(&self, parent_id: ObjectID) -> bool {
        match self.parent() {
            Some(obj_id) => obj_id == parent_id,
            None => false,
        }
    }

    /// The object's field key in the parent object
    pub fn field_key(&self) -> FieldKey {
        self.0
            .last()
            .cloned()
            .expect("Cannot get the field key of root object")
            .into()
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
        //Root object id is empty
        if hex_len == 0 {
            return Ok(Self::root());
        }
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

    pub fn try_from_annotated_move_struct_ref(value: &AnnotatedMoveStruct) -> Result<Self> {
        if value.value.len() != 1 {
            return Err(anyhow::anyhow!("Invalid ObjectID"));
        }
        let (field_name, field_value) = &value.value[0];
        debug_assert!(field_name.as_str() == "path");
        let path = match field_value {
            AnnotatedMoveValue::Vector(t, vector) => {
                debug_assert!(t == &TypeTag::Address);
                vector
                    .iter()
                    .map(|annotated_move_value| match annotated_move_value {
                        AnnotatedMoveValue::Address(addr) => Ok(*addr),
                        _ => Err(anyhow::anyhow!("Invalid ObjectID")),
                    })
                    .collect::<Result<Vec<AccountAddress>>>()?
            }
            _ => return Err(anyhow::anyhow!("Invalid ObjectID")),
        };
        Ok(ObjectID(path))
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
        ObjectID::try_from_annotated_move_struct_ref(&value)
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

pub fn custom_object_id<ID: Serialize>(id: &ID, struct_tag: &StructTag) -> ObjectID {
    //TODO raise error if the ID cannot be serialized
    let mut buffer = bcs::to_bytes(id).expect("ID to bcs should success");
    buffer.extend_from_slice(struct_tag.to_canonical_string().as_bytes());
    let struct_tag_hash = h256::sha3_256_of(&buffer);
    let child_part = AccountAddress::new(struct_tag_hash.0);
    ObjectID(vec![child_part])
}

pub fn custom_child_object_id<ID>(parent_id: ObjectID, id: &ID) -> ObjectID
where
    ID: MoveType + fmt::Debug + Serialize,
{
    //TODO raise error if the ID cannot be serialized
    //Child object id is the parent object id + child part
    //The child part same as the dynamic field key
    let child_part =
        FieldKey::derive(id).expect("ID to FieldKey should success, TODO: raise error");
    let ObjectID(mut parent_path) = parent_id;
    parent_path.push(child_part.into());
    ObjectID(parent_path)
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
pub type PackageObject = ObjectEntity<Package>;
pub type TimestampObject = ObjectEntity<Timestamp>;

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct ObjectMeta {
    pub id: ObjectID,
    pub owner: AccountAddress,
    pub flag: u8,
    pub state_root: Option<H256>,
    pub size: u64,
    /// The object created timestamp on chain
    pub created_at: u64,
    /// The object updated timestamp on chain
    /// Note: only the object value updated will update this timestamp
    /// The metadata updated or dynamic fields updated will not update this timestamp
    pub updated_at: u64,
    pub value_type: TypeTag,
}

impl ObjectMeta {
    pub fn new(
        id: ObjectID,
        owner: AccountAddress,
        flag: u8,
        state_root: Option<H256>,
        size: u64,
        created_at: u64,
        updated_at: u64,
        value_type: TypeTag,
    ) -> Self {
        Self {
            id,
            owner,
            flag,
            state_root,
            size,
            created_at,
            updated_at,
            value_type,
        }
    }

    pub fn genesis_root() -> Self {
        Self {
            id: ObjectID::root(),
            owner: MOVEOS_STD_ADDRESS,
            flag: SHARED_OBJECT_FLAG_MASK,
            state_root: None,
            size: 0,
            created_at: 0,
            updated_at: 0,
            value_type: Root::struct_tag().into(),
        }
    }

    pub fn genesis_meta(id: ObjectID, value_type: TypeTag) -> Self {
        Self {
            id,
            owner: SYSTEM_OWNER_ADDRESS,
            flag: 0,
            state_root: None,
            size: 0,
            created_at: 0,
            updated_at: 0,
            value_type,
        }
    }

    pub fn is_shared(&self) -> bool {
        self.flag & SHARED_OBJECT_FLAG_MASK == SHARED_OBJECT_FLAG_MASK
    }

    pub fn to_shared(&mut self) {
        self.flag |= SHARED_OBJECT_FLAG_MASK;
        self.owner = SYSTEM_OWNER_ADDRESS;
    }

    pub fn is_frozen(&self) -> bool {
        self.flag & FROZEN_OBJECT_FLAG_MASK == FROZEN_OBJECT_FLAG_MASK
    }

    pub fn to_frozen(&mut self) {
        self.flag |= FROZEN_OBJECT_FLAG_MASK;
        self.owner = SYSTEM_OWNER_ADDRESS;
    }

    pub fn is_genesis(&self) -> bool {
        match self.state_root {
            Some(state_root) => state_root == *GENESIS_STATE_ROOT,
            None => true,
        }
    }

    pub fn state_root(&self) -> H256 {
        self.state_root.unwrap_or_else(|| *GENESIS_STATE_ROOT)
    }

    pub fn is_system_owned(&self) -> bool {
        self.owner == SYSTEM_OWNER_ADDRESS
    }

    //If the object is system owned and not frozen or shared, it should be embeded in other struct
    pub fn is_embeded(&self) -> bool {
        self.is_system_owned() && !(self.is_frozen() || self.is_shared())
    }

    pub fn has_fields(&self) -> bool {
        let has_fields = self.size > 0;
        if !has_fields {
            debug_assert!(
                self.state_root.is_none() || self.state_root == Some(*GENESIS_STATE_ROOT)
            );
        }
        has_fields
    }

    pub fn update_state_root(&mut self, new_state_root: H256) {
        self.state_root = Some(new_state_root);
    }

    pub fn is_dynamic_field(&self) -> bool {
        is_dynamic_field_type(&self.value_type)
    }

    /// Exclude the DynamicField object
    pub fn is_object(&self) -> bool {
        !self.is_dynamic_field()
    }

    pub fn value_struct_tag(&self) -> &StructTag {
        match &self.value_type {
            TypeTag::Struct(struct_tag) => struct_tag,
            _ => panic!("The ObjectState must be Struct:{}", self.value_type),
        }
    }

    //TODO how to handle the resource display
    pub fn get_resource_struct_tag(&self) -> Option<&StructTag> {
        None
    }
}

/// The Entity of the Object<T>.
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ObjectEntity<T> {
    pub id: ObjectID,
    pub owner: AccountAddress,
    pub flag: u8,
    /// The state tree root of the object dynamic fields
    pub state_root: Option<H256>,
    pub size: u64,
    // The object created timestamp on chain
    pub created_at: u64,
    // The object updated timestamp on chain
    pub updated_at: u64,
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
            flag: SHARED_OBJECT_FLAG_MASK,
            state_root: Some(state_root),
            size,
            created_at: 0,
            updated_at: 0,
            value: Root {
                _placeholder: false,
            },
        }
    }
}

impl<T> ObjectEntity<T> {
    pub fn new(
        id: ObjectID,
        owner: AccountAddress,
        flag: u8,
        state_root: Option<H256>,
        size: u64,
        created_at: u64,
        updated_at: u64,
        value: T,
    ) -> ObjectEntity<T> {
        Self {
            id,
            owner,
            flag,
            state_root,
            size,
            created_at,
            updated_at,
            value,
        }
    }

    pub fn new_with_object_meta(meta: ObjectMeta, value: T) -> ObjectEntity<T> {
        Self {
            id: meta.id,
            owner: meta.owner,
            flag: meta.flag,
            state_root: meta.state_root,
            size: meta.size,
            created_at: meta.created_at,
            updated_at: meta.updated_at,
            value,
        }
    }

    pub fn state_root(&self) -> H256 {
        self.state_root.unwrap_or_else(|| *GENESIS_STATE_ROOT)
    }

    pub fn update_state_root(&mut self, new_state_root: H256) {
        self.state_root = Some(new_state_root);
    }

    pub fn is_shared(&self) -> bool {
        self.flag & SHARED_OBJECT_FLAG_MASK == SHARED_OBJECT_FLAG_MASK
    }

    pub fn to_shared(&mut self) {
        self.flag |= SHARED_OBJECT_FLAG_MASK;
    }

    pub fn is_frozen(&self) -> bool {
        self.flag & FROZEN_OBJECT_FLAG_MASK == FROZEN_OBJECT_FLAG_MASK
    }

    pub fn to_frozen(&mut self) {
        self.flag |= FROZEN_OBJECT_FLAG_MASK;
    }

    pub fn is_genesis(&self) -> bool {
        self.state_root() == *GENESIS_STATE_ROOT
    }

    pub fn is_system_owned(&self) -> bool {
        self.owner == SYSTEM_OWNER_ADDRESS
    }
}

impl<T> ObjectEntity<T>
where
    T: MoveStructState,
{
    pub fn metadata(&self) -> ObjectMeta {
        ObjectMeta {
            id: self.id.clone(),
            owner: self.owner,
            flag: self.flag,
            state_root: self.state_root,
            size: self.size,
            created_at: self.created_at,
            updated_at: self.updated_at,
            value_type: T::struct_tag().into(),
        }
    }

    pub fn into_state(self) -> ObjectState {
        let metadata = ObjectMeta {
            id: self.id,
            owner: self.owner,
            flag: self.flag,
            state_root: self.state_root,
            size: self.size,
            created_at: self.created_at,
            updated_at: self.updated_at,
            value_type: T::struct_tag().into(),
        };
        ObjectState::new(
            metadata,
            bcs::to_bytes(&self.value).expect("MoveState to bcs should success"),
        )
    }
}

impl ObjectEntity<TablePlaceholder> {
    pub fn new_table_object(id: ObjectID, state_root: H256, size: u64) -> TableObject {
        Self::new(
            id,
            SYSTEM_OWNER_ADDRESS,
            0u8,
            Some(state_root),
            size,
            0,
            0,
            TablePlaceholder::default(),
        )
    }
}

impl ObjectEntity<Account> {
    pub fn new_account_object(account: AccountAddress) -> AccountObject {
        Self::new(
            Account::account_object_id(account),
            account,
            0u8,
            None,
            0,
            0,
            0,
            Account::new(account, 0),
        )
    }
}

impl ObjectEntity<ModuleStore> {
    pub fn genesis_module_store() -> ModuleStoreObject {
        Self::new(
            ModuleStore::module_store_id(),
            MOVEOS_STD_ADDRESS,
            SHARED_OBJECT_FLAG_MASK,
            None,
            0,
            0,
            0,
            ModuleStore::default(),
        )
    }
}

impl ObjectEntity<Package> {
    pub fn new_package(address: &AccountAddress, owner: AccountAddress) -> PackageObject {
        Self::new(
            Package::package_id(address),
            owner,
            0u8,
            None,
            0,
            0,
            0,
            Package::default(),
        )
    }
}

impl ObjectEntity<Timestamp> {
    pub fn genesis_timestamp() -> TimestampObject {
        Self::new(
            Timestamp::object_id(),
            MOVEOS_STD_ADDRESS,
            0u8,
            None,
            0,
            0,
            0,
            Timestamp::default(),
        )
    }
}

impl<N, V> ObjectEntity<DynamicField<N, V>>
where
    N: MoveState + Serialize + fmt::Debug,
    V: MoveState,
{
    pub fn new_dynamic_field(parent_id: ObjectID, name: N, value: V) -> Self {
        let field_key = FieldKey::derive(&name).expect("FieldKey derive should success");
        let id = parent_id.child_id(field_key);
        Self::new(
            id,
            SYSTEM_OWNER_ADDRESS,
            0u8,
            None,
            0,
            0,
            0,
            DynamicField::new(name, value),
        )
    }
}

//TODO remove the RawObject
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

    pub fn struct_tag(&self) -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![self.value.struct_tag.clone().into()],
        }
    }

    // The output must consistent with ObjectEntity<T> into state result
    pub fn into_state(self) -> ObjectState {
        let value_type = TypeTag::Struct(Box::new(self.struct_tag()));
        let metadata = ObjectMeta::new(
            self.id,
            self.owner,
            self.flag,
            self.state_root,
            self.size,
            self.created_at,
            self.updated_at,
            value_type,
        );
        ObjectState::new(metadata, self.value.value)
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
            created_at: self.created_at,
            updated_at: self.updated_at,
            value,
        })
    }

    pub fn value_type(&self) -> TypeTag {
        TypeTag::Struct(Box::new(self.value.struct_tag.clone()))
    }
}

impl TryFrom<ObjectState> for RawObject {
    type Error = anyhow::Error;

    fn try_from(state: ObjectState) -> Result<Self> {
        state.into_raw_object()
    }
}

pub type AnnotatedObject = ObjectEntity<AnnotatedMoveStruct>;

impl AnnotatedObject {
    pub fn new_annotated_object(
        id: ObjectID,
        owner: AccountAddress,
        flag: u8,
        state_root: Option<H256>,
        size: u64,
        created_at: u64,
        updated_at: u64,
        value: AnnotatedMoveStruct,
    ) -> Self {
        Self {
            id,
            owner,
            flag,
            state_root,
            size,
            created_at,
            updated_at,
            value,
        }
    }

    pub fn new_from_annotated_struct(
        metadata: ObjectMeta,
        value_struct: AnnotatedMoveStruct,
    ) -> Result<Self> {
        Ok(Self::new_annotated_object(
            metadata.id,
            metadata.owner,
            metadata.flag,
            metadata.state_root,
            metadata.size,
            metadata.created_at,
            metadata.updated_at,
            value_struct,
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
    const STRUCT_NAME: &'static IdentStr = OBJECT_STRUCT_NAME;

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

pub fn is_object_struct(t: &StructTag) -> bool {
    Object::<PlaceholderStruct>::struct_tag_match_without_type_param(t)
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
            Some(H256::random()),
            0,
            0,
            0,
            object_value,
        );

        let object_state: ObjectState = object.clone().into_state();

        let runtime_value =
            Value::simple_deserialize(&object_state.value, &TestStruct::type_layout()).unwrap();
        let test_struct3 = TestStruct::from_runtime_value(runtime_value)?;
        assert_eq!(object.value, test_struct3);
        Ok(())
    }

    #[test]
    fn test_root_object() {
        let root_object = RootObjectEntity::genesis_root_object();
        let object_state = root_object.clone().into_state();

        let object = object_state.clone().into_object::<Root>().unwrap();
        assert_eq!(root_object, object);
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
        test_object_id_roundtrip(ObjectID::random());
    }

    #[test]
    fn test_named_object_id() {
        let struct_tag = StructTag {
            address: AccountAddress::from_str("0x2").unwrap(),
            module: ident_str!("object").to_owned(),
            name: ident_str!("Timestamp").to_owned(),
            type_params: vec![],
        };
        let timestamp_object_id = named_object_id(&struct_tag);
        //The object id generated by crates/rooch-framework-tests/tests/cases/timestamp/timestamp_test.move
        let object_id = ObjectID::from_str(
            "0x05921974509dbe44ab84328a625f4a6580a5f89dff3e4e2dec448cb2b1c7f5b9",
        )
        .unwrap();
        let timestamp_object_id2 = ObjectID::from_str("0x2::object::Timestamp").unwrap();
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
        let custom_object_id = custom_object_id(&id, &TestStruct::struct_tag());
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
        let root_object_id2 = ObjectID::from_bytes(bytes).unwrap();
        assert_eq!(root_object_id, root_object_id2);

        let id_hex = root_object_id.to_string();
        println!("root_object_id: {:?}", root_object_id);
        let root_object_id3 = ObjectID::from_str(&id_hex).unwrap();
        assert_eq!(root_object_id, root_object_id3);
    }
}
