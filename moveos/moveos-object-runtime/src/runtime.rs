// Copyright (c) RoochNetwork
// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::resolved_arg::ResolvedArg;
use better_any::{Tid, TidAble};
use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use move_core_types::{
    effects::Op,
    gas_algebra::NumBytes,
    ident_str,
    identifier::IdentStr,
    language_storage::{ModuleId, StructTag, TypeTag},
    value::{MoveStructLayout, MoveTypeLayout},
    vm_status::StatusCode,
};
use move_vm_runtime::native_functions::NativeContext;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    values::{GlobalValue, Reference, Struct, StructRef, Value},
};
use moveos_types::{
    addresses::MOVEOS_STD_ADDRESS,
    h256::H256,
    moveos_std::{
        move_module::{ModuleStore, MoveModule},
        object::{
            ModuleStoreObject, ObjectEntity, ObjectID, Root, RootObjectEntity, GENESIS_STATE_ROOT,
        },
    },
    state::{
        FieldChange, MoveStructState, MoveStructType, MoveType, NormalFieldChange, ObjectChange,
        State, StateChangeSet,
    },
    state_resolver::StatelessResolver,
};
use moveos_types::{
    moveos_std::{object, tx_context::TxContext},
    state::{KeyState, MoveState},
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::Arc,
};

/// Ensure the error codes in this file is consistent with the error code in object.move
pub const ERROR_ALREADY_EXISTS: u64 = 1;
pub const ERROR_NOT_FOUND: u64 = 2;
pub const ERROR_OBJECT_ALREADY_BORROWED: u64 = 7;
pub const ERROR_TYPE_MISMATCH: u64 = 10;
pub const ERROR_OBJECT_RUNTIME_ERROR: u64 = 14;

const FIELD_VALUE_STRUCT_NAME: &IdentStr = ident_str!("FieldValue");

/// The native Object runtime context extension. This needs to be attached to the NativeContextExtensions
/// value which is passed into session functions, so its accessible from natives of this
/// extension.
#[derive(Tid)]
pub struct ObjectRuntimeContext<'a> {
    resolver: &'a dyn StatelessResolver,
    object_runtime: Arc<RwLock<ObjectRuntime>>,
}

pub struct TxContextValue {
    value: GlobalValue,
}

/// A structure representing mutable data of the ObjectRuntimeContext. This is in a RefCell
/// of the overall context so we can mutate while still accessing the overall context.
pub struct ObjectRuntime {
    pub(crate) tx_context: TxContextValue,
    root: RuntimeObject,
    object_ref_in_args: BTreeMap<ObjectID, Value>,
    object_reference: BTreeMap<ObjectID, GlobalValue>,
}

/// A wrapper of Object dynamic field value, mirroring `FieldValue<V>` in `object.move`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FieldValue<V> {
    val: V,
}

pub struct RuntimeNormalField {
    key: KeyState,
    /// This is the Layout of FieldValue<V>
    value_layout: MoveTypeLayout,
    /// This is the TypeTag of FieldValue<V>
    value_type: TypeTag,
    /// This is the FieldValue<V> value in MoveVM memory
    value: GlobalValue,
}

/// A structure representing runtime field.
pub enum RuntimeField {
    None(KeyState),
    Object(RuntimeObject),
    Normal(RuntimeNormalField),
}

/// A structure representing a single runtime object.
pub struct RuntimeObject {
    id: ObjectID,
    /// This is the Layout of ObjectEntity<T>
    value_layout: MoveTypeLayout,
    /// This is the TypeTag of ObjectEntity<T>
    value_type: TypeTag,
    /// This is the ObjectEntity<T> value in MoveVM memory
    value: GlobalValue,
    /// The state root of the fields
    state_root: H256,
    fields: BTreeMap<KeyState, RuntimeField>,
}

pub trait TypeLayoutLoader {
    fn get_type_layout(&self, type_tag: &TypeTag) -> PartialVMResult<MoveTypeLayout>;
    fn type_to_type_layout(&self, ty: &Type) -> PartialVMResult<MoveTypeLayout>;
    fn type_to_type_tag(&self, ty: &Type) -> PartialVMResult<TypeTag>;
}

impl<'a, 'b> TypeLayoutLoader for NativeContext<'a, 'b> {
    fn get_type_layout(&self, type_tag: &TypeTag) -> PartialVMResult<MoveTypeLayout> {
        self.get_type_layout(type_tag).map_err(|e| e.to_partial())
    }
    fn type_to_type_layout(&self, ty: &Type) -> PartialVMResult<MoveTypeLayout> {
        self.type_to_type_layout(ty)?
            .ok_or_else(|| partial_extension_error("cannot determine type layout"))
    }
    fn type_to_type_tag(&self, ty: &Type) -> PartialVMResult<TypeTag> {
        self.type_to_type_tag(ty)
    }
}

// =========================================================================================
// Implementation of TxContextValue

impl TxContextValue {
    pub fn new(ctx: TxContext) -> Self {
        Self {
            value: GlobalValue::cached(ctx.to_runtime_value())
                .expect("Failed to cache the TxContext"),
        }
    }

    pub fn borrow_global(&self) -> PartialVMResult<Value> {
        self.value.borrow_global()
    }

    pub fn as_tx_context(&self) -> PartialVMResult<TxContext> {
        let value = self.value.borrow_global()?;
        let ctx_ref = value.value_as::<StructRef>()?;
        Ok(TxContext::from_runtime_value(ctx_ref.read_ref()?)
            .expect("Failed to convert Value to TxContext"))
    }

    pub fn into_inner(mut self) -> TxContext {
        let value = self
            .value
            .move_from()
            .expect("Failed to move value from GlobalValue");
        TxContext::from_runtime_value(value).expect("Failed to convert Value to TxContext")
    }
}

// =========================================================================================
// Implementation of ObjectRuntimeContext

impl<'a> ObjectRuntimeContext<'a> {
    /// Create a new instance of a object runtime context. This must be passed in via an
    /// extension into VM session functions.
    pub fn new(
        resolver: &'a dyn StatelessResolver,
        object_runtime: Arc<RwLock<ObjectRuntime>>,
    ) -> Self {
        //We need to init or load the module store before verify and execute tx
        object_runtime
            .write()
            .init_module_store(resolver)
            .expect("Failed to init module store");
        Self {
            resolver,
            object_runtime,
        }
    }

    pub fn object_runtime(&self) -> Arc<RwLock<ObjectRuntime>> {
        self.object_runtime.clone()
    }

    pub fn resolver(&self) -> &dyn StatelessResolver {
        self.resolver
    }
}

// =========================================================================================
// Implementation of FieldValue

impl<V> MoveStructType for FieldValue<V>
where
    V: MoveState,
{
    const ADDRESS: move_core_types::account_address::AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = object::MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = FIELD_VALUE_STRUCT_NAME;

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![V::type_tag()],
        }
    }
}

impl<V> MoveStructState for FieldValue<V>
where
    V: MoveState,
{
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![V::type_layout()])
    }

    fn from_runtime_value_struct(value: Struct) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut fields = value.unpack()?.collect::<Vec<Value>>();
        debug_assert!(fields.len() == 1, "Fields of FieldValue struct must be 1");
        let v = fields.pop().unwrap();
        Ok(FieldValue {
            val: V::from_runtime_value(v)?,
        })
    }
}

// =========================================================================================
// Implementation of ObjectRuntime

impl ObjectRuntime {
    pub fn new(tx_context: TxContext, root: RootObjectEntity) -> Self {
        if log::log_enabled!(log::Level::Trace) {
            log::trace!(
                "Init ObjectRuntime with tx_hash: {:?}, state_root: {}",
                tx_context.tx_hash(),
                root.state_root()
            );
        }
        Self {
            tx_context: TxContextValue::new(tx_context),
            root: RuntimeObject::load(
                root.id.clone(),
                RootObjectEntity::type_layout(),
                root.into_state(),
            )
            .expect("Load root object should success"),
            object_reference: Default::default(),
            object_ref_in_args: Default::default(),
        }
    }

    /// Initialize or load the module store Object into the ObjectRuntime.
    /// Because the module store is required when publishing genesis modules,
    /// So we can not initialize it in Move.
    pub fn init_module_store(&mut self, resolver: &dyn StatelessResolver) -> PartialVMResult<()> {
        let module_store_id = ModuleStore::module_store_id();
        let state = resolver
            .get_object_field_at(self.root.state_root, &module_store_id)
            .map_err(|e| {
                partial_extension_error(format!(
                    "Failed to resolve module store object state: {:?}",
                    e
                ))
            })?;
        let (state_root, global_value) = match state {
            Some(state) => {
                let obj = state.into_object::<ModuleStore>().map_err(|e| {
                    partial_extension_error(format!(
                        "Failed to resolve module store object: {:?}",
                        e
                    ))
                })?;
                let state_root = obj.state_root();
                let value = obj.to_runtime_value();
                // If we load the module store object, we should cache it
                (
                    state_root,
                    GlobalValue::cached(value)
                        .expect("Cache the ModuleStore Object should success"),
                )
            }
            None => {
                // If the module store object is not found, we should create a new one(before genesis).
                // Init none GlobalValue and move value to it, make the data status is dirty
                // The change will apart of the state change set
                let obj = ModuleStoreObject::genesis_module_store();
                let state_root = obj.state_root();
                let value = obj.to_runtime_value();
                let mut global_value = GlobalValue::none();
                global_value
                    .move_to(value)
                    .expect("Move value to GlobalValue none should success");
                (state_root, global_value)
            }
        };
        let module_store_runtime = RuntimeObject::init(
            module_store_id.clone(),
            ObjectEntity::<ModuleStore>::type_layout(),
            ObjectEntity::<ModuleStore>::type_tag(),
            global_value,
            state_root,
        )?;
        self.root.fields.insert(
            module_store_id.to_key(),
            RuntimeField::Object(module_store_runtime),
        );
        Ok(())
    }

    pub fn tx_context(&self) -> TxContext {
        self.tx_context
            .as_tx_context()
            .expect("Failed to get tx_context")
    }

    pub fn borrow_tx_context(&self) -> PartialVMResult<Value> {
        self.tx_context.borrow_global()
    }

    pub fn borrow_root(&self) -> PartialVMResult<Value> {
        self.root.borrow_value(ObjectEntity::<Root>::type_tag())
    }

    pub fn root(&self) -> RootObjectEntity {
        self.root
            .as_object_entity::<Root>()
            .expect("Failed to get root object")
    }

    pub fn add_to_tx_context<T: MoveState>(&mut self, value: T) -> PartialVMResult<()> {
        let mut tx_ctx = self.tx_context.as_tx_context()?;
        tx_ctx
            .add(value)
            .expect("Failed to add value to tx_context");
        self.tx_context = TxContextValue::new(tx_ctx);
        Ok(())
    }

    /// Load Object to the ObjectRuntime.
    pub fn load_object(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StatelessResolver,
        object_id: &ObjectID,
    ) -> PartialVMResult<(&mut RuntimeObject, Option<Option<NumBytes>>)> {
        if object_id.is_root() {
            Ok((&mut self.root, None))
        } else {
            let parent_id = object_id.parent().expect("expect parent id");
            let (parent, parent_load_gas) =
                self.load_object(layout_loader, resolver, &parent_id)?;
            let (obj, load_gas) = parent.load_object_field(layout_loader, resolver, object_id)?;
            let total_gas = sum_load_cost(parent_load_gas, load_gas);
            Ok((obj, total_gas))
        }
    }

    pub fn get_loaded_object(
        &self,
        object_id: &ObjectID,
    ) -> PartialVMResult<Option<&RuntimeObject>> {
        if object_id.is_root() {
            Ok(Some(&self.root))
        } else {
            let parent_id = object_id.parent().expect("expect parent id");
            let parent = self.get_loaded_object(&parent_id)?;
            match parent {
                Some(parent) => parent.get_loaded_object_field(object_id),
                None => Ok(None),
            }
        }
    }

    pub fn get_loaded_module(&self, module_id: &ModuleId) -> PartialVMResult<Option<MoveModule>> {
        let module_store_id = ModuleStore::module_store_id();
        let module_store_obj = self
            .get_loaded_object(&module_store_id)?
            .expect("module store object must exist");
        let key = KeyState::from_module_id(module_id);
        let module_field = module_store_obj.get_loaded_field(&key);
        match module_field {
            Some(field) => field.as_move_module(),
            None => Ok(None),
        }
    }

    pub fn load_module(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StatelessResolver,
        module_id: &ModuleId,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        let module_store_id = ModuleStore::module_store_id();
        match self.load_object(layout_loader, resolver, &module_store_id) {
            Ok((module_store_obj, _)) => {
                let key = KeyState::from_module_id(module_id);
                let (module_field, _loaded) =
                    module_store_obj.load_field(layout_loader, resolver, key)?;
                let move_module = module_field.as_move_module()?;
                Ok(move_module.map(|m| m.byte_codes))
            }
            Err(e) => {
                print!("load_module error: {:?}", e);
                // convert the error to StatusCode::MISSING_DATA if the module is not found
                if e.major_status() == StatusCode::MISSING_DATA {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn publish_module(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StatelessResolver,
        module_id: &ModuleId,
        blob: Vec<u8>,
        is_republishing: bool,
    ) -> PartialVMResult<()> {
        let module_store_id = ModuleStore::module_store_id();
        let (module_store_obj, _) = self.load_object(layout_loader, resolver, &module_store_id)?;
        let key = KeyState::from_module_id(module_id);
        let (module_field, _) = module_store_obj.load_field(layout_loader, resolver, key)?;

        let value_type = FieldValue::<MoveModule>::type_tag();
        let value_layout = FieldValue::<MoveModule>::type_layout();

        let move_module = MoveModule::new(blob);
        let field_value = FieldValue { val: move_module };
        let runtime_field_value = field_value.to_runtime_value();
        if is_republishing {
            let _old_value = module_field.move_from(value_type.clone())?;
        }

        module_field.move_to(runtime_field_value, value_layout, value_type)?;

        Ok(())
    }

    pub fn exists_loaded_module(&self, module_id: &ModuleId) -> PartialVMResult<bool> {
        self.get_loaded_module(module_id).map(|m| m.is_some())
    }

    pub fn load_object_reference(&mut self, object_id: &ObjectID) -> VMResult<()> {
        self.object_reference
            .entry(object_id.clone())
            .or_insert_with(|| {
                //TODO we should load the ObjectEntity<T> from the resolver
                //Then cache the Object<T>
                let object_id_value = object_id.to_runtime_value();
                GlobalValue::cached(Value::struct_(Struct::pack(vec![object_id_value])))
                    .expect("Failed to cache the Struct")
            });
        Ok(())
    }

    /// Borrow &Object<T> or &mut Object<T>
    pub fn borrow_object_reference(&mut self, object_id: &ObjectID) -> VMResult<Value> {
        let gv = self.object_reference.get(object_id).ok_or_else(|| {
            PartialVMError::new(StatusCode::STORAGE_ERROR).finish(Location::Undefined)
        })?;

        if gv.reference_count() >= 2 {
            // We raise an error if the object is already borrowed
            // Use the error code in object.move for easy debugging
            return Err(PartialVMError::new(StatusCode::ABORTED)
                .with_sub_status(ERROR_OBJECT_ALREADY_BORROWED)
                .with_message(format!("Object {} already borrowed", object_id))
                .finish(Location::Module(object::MODULE_ID.clone())));
        }

        gv.borrow_global()
            .map_err(|e| e.finish(Location::Undefined))
    }

    pub fn load_arguments(&mut self, resovled_args: &[ResolvedArg]) -> VMResult<()> {
        for resolved_arg in resovled_args {
            if let ResolvedArg::Object(object_arg) = resolved_arg {
                let object_id = object_arg.object_id();
                self.load_object_reference(object_id)?;
                let ref_value = self.borrow_object_reference(object_id)?;
                //We cache the object reference in the object_ref_in_args
                //Ensure the reference count and the object can not be borrowed in Move
                self.object_ref_in_args.insert(object_id.clone(), ref_value);
            }
        }
        Ok(())
    }

    // into inner
    fn into_inner(self) -> (TxContext, RuntimeObject) {
        let ObjectRuntime {
            tx_context,
            root,
            object_reference: _,
            object_ref_in_args: _,
        } = self;
        (tx_context.into_inner(), root)
    }

    pub fn into_change_set(self) -> PartialVMResult<(TxContext, StateChangeSet)> {
        let (tx_context, root) = self.into_inner();
        let root_entity = root.as_object_entity::<Root>()?;
        let root_change = root.into_change()?;
        let mut changes = BTreeMap::new();
        if let Some(root_change) = root_change {
            for (k, field_change) in root_change.fields {
                let obj_change = field_change
                    .into_object_change()
                    .expect("root object's field must be object");
                changes.insert(
                    k.as_object_id()
                        .expect("object change's key must be ObjectID"),
                    obj_change,
                );
            }
        }
        let change_set = StateChangeSet {
            state_root: root_entity.state_root(),
            global_size: root_entity.size,
            changes,
        };
        Ok((tx_context, change_set))
    }
}

// =========================================================================================
// Implementation of RuntimeField

impl RuntimeNormalField {
    pub fn load(
        key: KeyState,
        value_layout: MoveTypeLayout,
        state: State,
    ) -> PartialVMResult<Self> {
        let value = deserialize(&value_layout, state.value.as_slice())?;
        //the state only contains V of FieldValue<T>, so we need wrap there.
        let wraped_value = Value::struct_(Struct::pack(vec![value]));
        let wraped_layout = MoveTypeLayout::Struct(MoveStructLayout::new(vec![value_layout]));
        let wraped_type = TypeTag::Struct(Box::new(StructTag {
            address: MOVEOS_STD_ADDRESS,
            module: object::MODULE_NAME.to_owned(),
            name: FIELD_VALUE_STRUCT_NAME.to_owned(),
            type_params: vec![state.value_type],
        }));
        Ok(Self {
            key,
            value_layout: wraped_layout,
            value_type: wraped_type,
            value: GlobalValue::cached(wraped_value)?,
        })
    }

    pub fn init(
        key: KeyState,
        value_layout: MoveTypeLayout,
        value_type: TypeTag,
        value: GlobalValue,
    ) -> PartialVMResult<Self> {
        Ok(Self {
            key,
            value_layout,
            value_type,
            value,
        })
    }

    pub fn key(&self) -> &KeyState {
        &self.key
    }

    pub fn move_to(
        &mut self,
        val: Value,
        _value_layout: MoveTypeLayout,
        value_type: TypeTag,
    ) -> PartialVMResult<()> {
        if self.value.exists()? {
            return Err(PartialVMError::new(StatusCode::RESOURCE_ALREADY_EXISTS)
                .with_message("Field already exists".to_string()));
        }
        check_type(&self.value_type, &value_type)?;
        self.value.move_to(val).map_err(|(e, _)| e)
    }

    pub fn borrow_value(&self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        check_type(&self.value_type, &expect_value_type)?;
        self.value.borrow_global()
    }

    pub fn move_from(&mut self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        check_type(&self.value_type, &expect_value_type)?;
        self.value.move_from()
    }

    pub fn as_move_module(&self) -> PartialVMResult<Option<MoveModule>> {
        if !self.value.exists()? {
            Ok(None)
        } else {
            let field_runtime_value_ref = self.borrow_value(FieldValue::<MoveModule>::type_tag())?;
            let field_runtime_value = field_runtime_value_ref
                .value_as::<Reference>()?
                .read_ref()?;
            let field_value = FieldValue::<MoveModule>::from_runtime_value(field_runtime_value)
                .map_err(|e| {
                    partial_extension_error(format!(
                        "expect FieldValue<MoveModule>, but got {:?}",
                        e
                    ))
                })?;
            Ok(Some(field_value.val))
        }
    }

    pub fn into_effect(self) -> (MoveTypeLayout, TypeTag, Option<Op<Value>>) {
        let op = self.value.into_effect();
        // we need to unwrap the FieldValue<V> to V
        let unwraped_op = op.map(|op| {
            op.map(|val| {
                let field_value_struct = val.value_as::<Struct>().expect("expect struct value");
                let mut fields = field_value_struct
                    .unpack()
                    .expect("unpack struct should success");
                fields
                    .next()
                    .expect("FieldValue<V> should have one field of type V")
            })
        });
        let unwraped_layout = match self.value_layout {
            MoveTypeLayout::Struct(layout) => layout
                .into_fields()
                .pop()
                .expect("expect FieldValue<V> must have one field"),
            _ => unreachable!("expect FieldValue<V> to be a struct"),
        };
        let unwraped_type = match self.value_type {
            TypeTag::Struct(mut tag) => tag.type_params.pop().unwrap_or_else(|| {
                panic!(
                    "expect FieldValue<V> must have one type param, value_type:{:?}",
                    tag
                )
            }),
            _ => unreachable!("expect FieldValue<V> to be a struct"),
        };
        (unwraped_layout, unwraped_type, unwraped_op)
    }

    pub fn into_change(self) -> PartialVMResult<Option<NormalFieldChange>> {
        let (layout, value_type, op) = self.into_effect();
        Ok(match op {
            Some(op) => {
                let change = op.and_then(|v| {
                    let bytes = serialize(&layout, &v)?;
                    let state = State::new(bytes, value_type);
                    Ok(state)
                })?;
                Some(NormalFieldChange { op: change })
            }
            None => None,
        })
    }
}

impl RuntimeField {
    pub fn field_type(&self) -> String {
        match self {
            RuntimeField::None(_) => "None".to_string(),
            RuntimeField::Object(f) => f.value_type.to_string(),
            RuntimeField::Normal(f) => f.value_type.to_string(),
        }
    }

    /// Load field from state
    pub fn load(
        key: KeyState,
        value_layout: MoveTypeLayout,
        state: State,
    ) -> PartialVMResult<Self> {
        if object::is_object_entity_type(&state.value_type) {
            let object = RuntimeObject::load(
                key.as_object_id().map_err(|e| {
                    partial_extension_error(format!("expect object id,but got {:?}, {:?}", key, e))
                })?,
                value_layout,
                state,
            )?;
            Ok(RuntimeField::Object(object))
        } else {
            let normal = RuntimeNormalField::load(key, value_layout, state)?;
            Ok(RuntimeField::Normal(normal))
        }
    }

    /// Init a field with value
    pub fn init(
        key: KeyState,
        value_layout: MoveTypeLayout,
        value_type: TypeTag,
        value: Value,
    ) -> PartialVMResult<Self> {
        // Init none GlobalValue and move value to it, make the data status is dirty
        let mut global_value = GlobalValue::none();
        global_value
            .move_to(value)
            .expect("Move value to GlobalValue none should success");

        Ok(if object::is_object_entity_type(&value_type) {
            //The object field is new, so the state_root is the genesis state root
            let state_root = *GENESIS_STATE_ROOT;
            let object = RuntimeObject::init(
                key.as_object_id().map_err(|e| {
                    partial_extension_error(format!("expect object id, but got {:?}, {:?}", key, e))
                })?,
                value_layout,
                value_type,
                global_value,
                state_root,
            )?;
            RuntimeField::Object(object)
        } else {
            let normal = RuntimeNormalField::init(key, value_layout, value_type, global_value)?;
            RuntimeField::Normal(normal)
        })
    }

    pub fn none(key: KeyState) -> Self {
        RuntimeField::None(key)
    }

    pub fn exists(&self) -> PartialVMResult<bool> {
        match self {
            RuntimeField::None(_) => Ok(false),
            RuntimeField::Object(obj) => Ok(obj.value.exists()?),
            RuntimeField::Normal(normal) => Ok(normal.value.exists()?),
        }
    }

    pub fn exists_with_type(&self, expect_value_type: TypeTag) -> PartialVMResult<bool> {
        match self {
            RuntimeField::None(_) => Ok(false),
            RuntimeField::Object(obj) => {
                Ok(obj.value.exists()? && obj.value_type == expect_value_type)
            }
            RuntimeField::Normal(normal) => {
                Ok(normal.value.exists()? && normal.value_type == expect_value_type)
            }
        }
    }

    pub fn move_to(
        &mut self,
        val: Value,
        value_layout: MoveTypeLayout,
        value_type: TypeTag,
    ) -> PartialVMResult<()> {
        match self {
            RuntimeField::None(key) => {
                *self = Self::init(key.clone(), value_layout, value_type, val)?;
                Ok(())
            }
            RuntimeField::Object(obj) => obj.move_to(val, value_layout, value_type),
            RuntimeField::Normal(normal) => normal.move_to(val, value_layout, value_type),
        }
    }

    pub fn borrow_value(&self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        match self {
            RuntimeField::None(_) => Err(PartialVMError::new(StatusCode::MISSING_DATA)
                .with_message(format!(
                    "Cannot borrow value of None as type {}",
                    expect_value_type
                ))),
            RuntimeField::Object(obj) => obj.borrow_value(expect_value_type),
            RuntimeField::Normal(normal) => normal.borrow_value(expect_value_type),
        }
    }

    pub fn move_from(&mut self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        match self {
            RuntimeField::None(_) => Err(PartialVMError::new(StatusCode::MISSING_DATA)
                .with_message(format!(
                    "Cannot move value of unknown type as type {}",
                    expect_value_type
                ))),
            RuntimeField::Object(obj) => obj.move_from(expect_value_type),
            RuntimeField::Normal(normal) => normal.move_from(expect_value_type),
        }
    }

    pub fn as_move_module(&self) -> PartialVMResult<Option<MoveModule>> {
        match self {
            RuntimeField::Normal(normal) => normal.as_move_module(),
            RuntimeField::None(_) => Ok(None),
            _ => Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                .with_message("Not Module Field".to_string())),
        }
    }

    pub fn into_effect(self) -> Option<(MoveTypeLayout, TypeTag, Op<Value>)> {
        match self {
            RuntimeField::None(_) => None,
            RuntimeField::Object(obj) => {
                let (layout, value_type, op) = obj.into_effect();
                op.map(|op| (layout, value_type, op))
            }
            RuntimeField::Normal(normal) => {
                let (layout, value_type, op) = normal.into_effect();
                op.map(|op| (layout, value_type, op))
            }
        }
    }

    pub fn into_change(self) -> PartialVMResult<Option<FieldChange>> {
        match self {
            RuntimeField::None(_) => Ok(None),
            RuntimeField::Object(obj) => obj.into_change().map(|op| op.map(FieldChange::Object)),
            RuntimeField::Normal(normal) => {
                normal.into_change().map(|op| op.map(FieldChange::Normal))
            }
        }
    }
}

// =========================================================================================
// Implementation of RuntimeObject

impl RuntimeObject {
    pub fn id(&self) -> &ObjectID {
        &self.id
    }

    pub fn load(id: ObjectID, value_layout: MoveTypeLayout, state: State) -> PartialVMResult<Self> {
        let raw_obj = state
            .as_raw_object()
            .map_err(|e| partial_extension_error(format!("expect raw object, but got {:?}", e)))?;
        let value = deserialize(&value_layout, state.value.as_slice())?;
        Ok(Self {
            id,
            value_layout,
            value_type: state.value_type,
            value: GlobalValue::cached(value)?,
            state_root: raw_obj.state_root(),
            fields: Default::default(),
        })
    }

    pub fn init(
        id: ObjectID,
        value_layout: MoveTypeLayout,
        value_type: TypeTag,
        value: GlobalValue,
        state_root: H256,
    ) -> PartialVMResult<Self> {
        Ok(Self {
            id,
            value_layout,
            value_type,
            value,
            state_root,
            fields: Default::default(),
        })
    }

    pub fn move_to(
        &mut self,
        val: Value,
        _value_layout: MoveTypeLayout,
        value_type: TypeTag,
    ) -> PartialVMResult<()> {
        if self.value.exists()? {
            return Err(PartialVMError::new(StatusCode::RESOURCE_ALREADY_EXISTS)
                .with_message("Object Field already exists".to_string()));
        }
        check_type(&self.value_type, &value_type)?;
        self.value.move_to(val).map_err(|(e, _)| e)
    }

    pub fn borrow_value(&self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        check_type(&self.value_type, &expect_value_type)?;
        self.value.borrow_global()
    }

    pub fn move_from(&mut self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        check_type(&self.value_type, &expect_value_type)?;
        self.value.move_from()
    }

    /// Load a field from the object. If the field not exists, init a None field.
    pub fn load_field(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StatelessResolver,
        key: KeyState,
    ) -> PartialVMResult<(&mut RuntimeField, Option<Option<NumBytes>>)> {
        self.load_field_with_layout_fn(resolver, key, |value_type| {
            layout_loader.get_type_layout(value_type)
        })
    }

    pub fn get_loaded_field(&self, key: &KeyState) -> Option<&RuntimeField> {
        self.fields.get(key)
    }

    pub fn load_object_field(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StatelessResolver,
        object_id: &ObjectID,
    ) -> PartialVMResult<(&mut RuntimeObject, Option<Option<NumBytes>>)> {
        let (field, loaded) = self.load_field(layout_loader, resolver, object_id.to_key())?;
        match field {
            RuntimeField::Object(obj) => Ok((obj, loaded)),
            RuntimeField::None(_) => Err(PartialVMError::new(StatusCode::MISSING_DATA)
                .with_message(format!("Can not load Object with id: {}", object_id))),
            _ => Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                .with_message(format!("Not Object Field with key {}", object_id))),
        }
    }

    pub fn get_loaded_object_field(
        &self,
        object_id: &ObjectID,
    ) -> PartialVMResult<Option<&RuntimeObject>> {
        let field = self.get_loaded_field(&object_id.to_key());
        match field {
            Some(RuntimeField::Object(obj)) => Ok(Some(obj)),
            Some(RuntimeField::None(_)) => Ok(None),
            None => Ok(None),
            Some(RuntimeField::Normal(field)) => {
                debug_assert!(
                    false,
                    "expect object field, but got {:?}, this should not happend",
                    field.value_type
                );
                Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                    .with_message(format!("Not Object Field with key {}", object_id)))
            }
        }
    }

    pub fn load_field_with_layout_fn(
        &mut self,
        resolver: &dyn StatelessResolver,
        key: KeyState,
        f: impl FnOnce(&TypeTag) -> PartialVMResult<MoveTypeLayout>,
    ) -> PartialVMResult<(&mut RuntimeField, Option<Option<NumBytes>>)> {
        Ok(match self.fields.entry(key.clone()) {
            Entry::Vacant(entry) => {
                let (tv, loaded) =
                    match resolver
                        .get_field_at(self.state_root, &key)
                        .map_err(|err| {
                            partial_extension_error(format!(
                                "remote object resolver failure: {}",
                                err
                            ))
                        })? {
                        Some(state) => {
                            let value_layout = f(&state.value_type)?;
                            let state_bytes_len = state.value.len() as u64;
                            (
                                RuntimeField::load(key, value_layout, state)?,
                                Some(NumBytes::new(state_bytes_len)),
                            )
                        }
                        None => (RuntimeField::none(key), None),
                    };
                (entry.insert(tv), Some(loaded))
            }
            Entry::Occupied(entry) => (entry.into_mut(), None),
        })
    }

    pub fn into_effect(self) -> (MoveTypeLayout, TypeTag, Option<Op<Value>>) {
        let op = self.value.into_effect();
        (self.value_layout, self.value_type, op)
    }

    pub fn into_change(self) -> PartialVMResult<Option<ObjectChange>> {
        let op = self.value.into_effect();
        let change = match op {
            Some(op) => {
                let change = op.and_then(|v| {
                    let bytes = serialize(&self.value_layout, &v)?;
                    let state = State::new(bytes, self.value_type);
                    Ok(state)
                })?;
                Some(change)
            }
            None => None,
        };
        let mut fields_change = BTreeMap::new();
        for (key, field) in self.fields.into_iter() {
            let field_change = field.into_change()?;
            if let Some(change) = field_change {
                fields_change.insert(key, change);
            }
        }
        if change.is_none() && fields_change.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ObjectChange {
                op: change,
                fields: fields_change,
            }))
        }
    }

    pub fn as_object_entity<T: MoveStructState>(&self) -> PartialVMResult<ObjectEntity<T>> {
        let obj_value = self.value.borrow_global()?;
        let value_ref = obj_value.value_as::<StructRef>()?;
        ObjectEntity::<T>::from_runtime_value(value_ref.read_ref()?)
            .map_err(|_e| partial_extension_error("Convert value to ObjectEntity failed"))
    }
}

fn partial_extension_error(msg: impl ToString) -> PartialVMError {
    log::debug!("PartialVMError: {}", msg.to_string());
    PartialVMError::new(StatusCode::VM_EXTENSION_ERROR).with_message(msg.to_string())
}

fn check_type(actual_type: &TypeTag, expect_type: &TypeTag) -> PartialVMResult<()> {
    if expect_type != actual_type {
        return Err(
            PartialVMError::new(StatusCode::TYPE_MISMATCH).with_message(format!(
                "Field state type {}, but get type {}",
                actual_type, expect_type
            )),
        );
    }
    Ok(())
}

fn sum_load_cost(
    first_loaded: Option<Option<NumBytes>>,
    second_loaded: Option<Option<NumBytes>>,
) -> Option<Option<NumBytes>> {
    match (first_loaded, second_loaded) {
        (Some(Some(first)), Some(Some(second))) => Some(Some(first + second)),
        (Some(Some(first)), None) => Some(Some(first)),
        (None, Some(Some(second))) => Some(Some(second)),
        (Some(None), _) | (_, Some(None)) => Some(None),
        _ => None,
    }
}

pub fn serialize(layout: &MoveTypeLayout, val: &Value) -> PartialVMResult<Vec<u8>> {
    val.simple_serialize(layout).ok_or_else(|| {
        partial_extension_error(format!(
            "cannot serialize object field or value, layout:{:?}, val:{:?}",
            layout, val
        ))
    })
}

pub fn deserialize(layout: &MoveTypeLayout, bytes: &[u8]) -> PartialVMResult<Value> {
    let value = Value::simple_deserialize(bytes, layout).ok_or_else(|| {
        partial_extension_error(format!(
            "cannot deserialize object field or value, layout:{:?}, bytes:{:?}",
            layout,
            hex::encode(bytes)
        ))
    })?;
    Ok(value)
}
