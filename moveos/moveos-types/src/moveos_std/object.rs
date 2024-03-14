// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::table::TablePlaceholder;
use crate::moveos_std::account::Account;
use crate::moveos_std::move_module::ModuleStore;
use crate::moveos_std::object_id::ObjectID;
use crate::state_resolver;
use crate::{
    addresses::MOVEOS_STD_ADDRESS,
    state::{MoveState, MoveStructState, MoveStructType, State},
};
use anyhow::{bail, ensure, Result};
use move_core_types::language_storage::ModuleId;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::{StructTag, TypeTag},
    value::{MoveStructLayout, MoveTypeLayout},
};
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use once_cell::sync::Lazy;
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use smt::SPARSE_MERKLE_PLACEHOLDER_HASH;

pub const MODULE_NAME: &IdentStr = ident_str!("object");
pub static MODULE_ID: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(MOVEOS_STD_ADDRESS, MODULE_NAME.to_owned()));
pub const OBJECT_ENTITY_STRUCT_NAME: &IdentStr = ident_str!("ObjectEntity");
//TODO rename to ROOT_OBJECT_ID
pub const GLOBAL_OBJECT_STORAGE_HANDLE: ObjectID = state_resolver::GLOBAL_OBJECT_STORAGE_HANDLE;

// New table's state_root should be the place holder hash.
pub static GENESIS_STATE_ROOT: Lazy<H256> = Lazy::new(|| *SPARSE_MERKLE_PLACEHOLDER_HASH);

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
            id: self.id,
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

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct RawData {
    pub struct_tag: StructTag,
    pub value: Vec<u8>,
}

impl RawObject {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = OBJECT_ENTITY_STRUCT_NAME;

    pub fn from_bytes(bytes: &[u8], struct_tag: StructTag) -> Result<Self> {
        ensure!(
            bytes.len() > ObjectID::LENGTH + AccountAddress::LENGTH + AccountAddress::LENGTH,
            "Invalid bytes length"
        );

        let id: ObjectID = bcs::from_bytes(&bytes[..ObjectID::LENGTH])?;
        let owner: AccountAddress =
            bcs::from_bytes(&bytes[ObjectID::LENGTH..ObjectID::LENGTH + AccountAddress::LENGTH])?;
        let flag = bytes[ObjectID::LENGTH + AccountAddress::LENGTH
            ..ObjectID::LENGTH + AccountAddress::LENGTH + 1][0];
        let state_root: AccountAddress = bcs::from_bytes(
            &bytes[ObjectID::LENGTH + AccountAddress::LENGTH + 1
                ..ObjectID::LENGTH + AccountAddress::LENGTH + 1 + AccountAddress::LENGTH],
        )?;
        let size: u64 = bcs::from_bytes(
            &bytes[ObjectID::LENGTH + AccountAddress::LENGTH + 1 + AccountAddress::LENGTH
                ..ObjectID::LENGTH + AccountAddress::LENGTH + 1 + AccountAddress::LENGTH + 8],
        )?;
        let value = bytes
            [ObjectID::LENGTH + AccountAddress::LENGTH + 1 + AccountAddress::LENGTH + 8..]
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use move_vm_types::values::Value;

    use super::*;

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
        let object_id = ObjectID::new(crate::h256::H256::random().into());
        let object = ObjectEntity::new(
            object_id,
            AccountAddress::random(),
            0u8,
            H256::random(),
            0,
            object_value,
        );

        let raw_object: RawObject =
            RawObject::from_bytes(&object.to_bytes()?, TestStruct::struct_tag())?;

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
}
