// Copyright (c) RoochNetwork
// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    resolved_arg::{ObjectArg, ResolvedArg},
    runtime_object::RuntimeObject,
    tx_context::TxContextValue,
    TypeLayoutLoader,
};
use better_any::{Tid, TidAble};
use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use move_core_types::{
    account_address::AccountAddress, gas_algebra::NumBytes, language_storage::ModuleId,
    value::MoveTypeLayout, vm_status::StatusCode,
};
use move_vm_types::values::{StructRef, Value};
use moveos_types::{
    move_std::string::MoveString,
    moveos_std::timestamp::Timestamp,
    state::{FieldKey, ObjectChange, ObjectState},
};
use moveos_types::{moveos_std::object::TimestampObject, state::StateChangeSet};
use moveos_types::{
    moveos_std::{
        module_store::{ModuleStore, Package},
        move_module::MoveModule,
        object::{DynamicField, ModuleStoreObject, ObjectID, PackageObject, Root},
    },
    state::MoveType,
    state_resolver::StatelessResolver,
};
use moveos_types::{
    moveos_std::{object, tx_context::TxContext},
    state::MoveState,
};
use parking_lot::RwLock;
use std::{collections::BTreeMap, sync::Arc};
use tracing::debug;

/// Ensure the error codes in this file is consistent with the error code in object.move
pub const ERROR_ALREADY_EXISTS: u64 = 1;
pub const ERROR_NOT_FOUND: u64 = 2;
pub const ERROR_INVALID_OWNER_ADDRESS: u64 = 3;
pub const ERROR_OBJECT_OWNER_NOT_MATCH: u64 = 4;
pub const ERROR_OBJECT_NOT_SHARED: u64 = 5;
pub const ERROR_OBJECT_IS_BOUND: u64 = 6;
pub const ERROR_OBJECT_ALREADY_BORROWED: u64 = 7;
pub const ERROR_FIELDS_NOT_EMPTY: u64 = 8;
pub const ERROR_OBJECT_FROZEN: u64 = 9;
pub const ERROR_TYPE_MISMATCH: u64 = 10;
pub const ERROR_CHILD_OBJECT_TOO_DEEP: u64 = 11;
pub const ERROR_WITHOUT_PARENT: u64 = 12;
pub const ERROR_PARENT_NOT_MATCH: u64 = 13;
pub const ERROR_OBJECT_RUNTIME_ERROR: u64 = 14;
pub const ERROR_OBJECT_ALREADY_TAKEN_OUT_OR_EMBEDED: u64 = 15;

/// The native Object runtime context extension. This needs to be attached to the NativeContextExtensions
/// value which is passed into session functions, so its accessible from natives of this
/// extension.
#[derive(Tid)]
pub struct ObjectRuntimeContext<'a> {
    resolver: &'a dyn StatelessResolver,
    object_runtime: Arc<RwLock<ObjectRuntime>>,
}

impl<'a> ObjectRuntimeContext<'a> {
    /// Create a new instance of a object runtime context. This must be passed in via an
    /// extension into VM session functions.
    pub fn new(
        resolver: &'a dyn StatelessResolver,
        object_runtime: Arc<RwLock<ObjectRuntime>>,
    ) -> Self {
        //We need to init or load the module store and timestamp store before verify and execute tx
        object_runtime
            .write()
            .init_module_store(resolver)
            .expect("Failed to init module store");
        object_runtime
            .write()
            .init_timestamp_store(resolver)
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

/// A structure representing mutable data of the ObjectRuntimeContext. This is in a RefCell
/// of the overall context so we can mutate while still accessing the overall context.
pub struct ObjectRuntime {
    pub(crate) tx_context: TxContextValue,
    pub(crate) root: RuntimeObject,
    pub(crate) object_pointer_in_args: BTreeMap<ObjectID, Value>,
}

impl ObjectRuntime {
    pub fn new(tx_context: TxContext, root: ObjectState) -> Self {
        if log::log_enabled!(log::Level::Trace) {
            tracing::trace!(
                "Init ObjectRuntime with tx_hash: {:?}, state_root: {}",
                tx_context.tx_hash(),
                root.state_root()
            );
        }
        Self {
            tx_context: TxContextValue::new(tx_context),
            root: RuntimeObject::load(Root::type_layout(), root)
                .expect("Load root object should success"),
            object_pointer_in_args: Default::default(),
        }
    }

    /// Initialize or load the module store Object into the ObjectRuntime.
    /// Because the module store is required when publishing genesis modules,
    /// So we can not initialize it in Move.
    pub fn init_module_store(&mut self, resolver: &dyn StatelessResolver) -> PartialVMResult<()> {
        let module_store_id = ModuleStore::module_store_id();
        let field_key = module_store_id.field_key();
        let state = resolver
            .get_field_at(self.root.state_root()?, &field_key)
            .map_err(|e| {
                partial_extension_error(format!(
                    "Failed to resolve module store object state: {:?}",
                    e
                ))
            })?;
        let module_store_rt_obj = match state {
            Some(obj_state) => RuntimeObject::load(ModuleStore::type_layout(), obj_state)?,
            None => {
                //TODO Put ModuleStore Object to genesis config.
                // If the module store object is not found, we should create a new one(before genesis).
                // Init none GlobalValue and move value to it, make the data status is dirty
                // The change will apart of the state change set
                let obj = ModuleStoreObject::genesis_module_store();
                let value_type = ModuleStore::type_tag();
                let value_layout = ModuleStore::type_layout();
                let value = obj.value.to_runtime_value();
                let mut rt_obj = RuntimeObject::none(obj.id);
                rt_obj.move_to(value, value_type, value_layout)?;
                rt_obj.rt_meta.to_shared()?;
                rt_obj
            }
        };
        debug!(
            "Init module store object with state_root: {}",
            module_store_rt_obj.rt_meta.state_root()?
        );

        self.root.fields.insert(field_key, module_store_rt_obj);
        Ok(())
    }

    /// Initialize or load the timestamp store Object into the ObjectRuntime.
    /// Because the timestamp store is required when execute unit test,
    /// So we initialize it in the ObjectRuntime.
    pub fn init_timestamp_store(
        &mut self,
        resolver: &dyn StatelessResolver,
    ) -> PartialVMResult<()> {
        let timestamp_id = Timestamp::object_id();
        let field_key = timestamp_id.field_key();
        let state = resolver
            .get_field_at(self.root.state_root()?, &field_key)
            .map_err(|e| {
                partial_extension_error(format!(
                    "Failed to resolve timestamp object state: {:?}",
                    e
                ))
            })?;
        let timestamp_rt_obj = match state {
            Some(obj_state) => RuntimeObject::load(Timestamp::type_layout(), obj_state)?,
            None => {
                // If the timestamp object is not found, we should create a new one(before genesis).
                // Init none GlobalValue and move value to it, make the data status is dirty
                // The change will apart of the state change set
                let obj = TimestampObject::genesis_timestamp();
                let value_type = Timestamp::type_tag();
                let value_layout = Timestamp::type_layout();
                let value = obj.value.to_runtime_value();

                let mut rt_obj = RuntimeObject::none(obj.id);
                rt_obj.move_to(value, value_type, value_layout)?;
                rt_obj.rt_meta.to_shared()?;
                rt_obj
            }
        };
        debug!(
            "Init timestamp object with state_root: {}",
            timestamp_rt_obj.rt_meta.state_root()?
        );
        self.root.fields.insert(field_key, timestamp_rt_obj);
        Ok(())
    }

    fn load_or_create_package_object<'a>(
        module_store_obj: &'a mut RuntimeObject,
        layout_loader: &'a dyn TypeLayoutLoader,
        resolver: &'a dyn StatelessResolver,
        address: &'a AccountAddress,
        package_owner: AccountAddress,
    ) -> PartialVMResult<(&'a mut RuntimeObject, bool)> {
        let package_field_key = Package::derive_package_key(address);
        let (package_obj, _) =
            module_store_obj.load_field(layout_loader, resolver, package_field_key)?;

        let mut new_package = false;
        if !package_obj.exists()? {
            let obj = PackageObject::new_package(address, package_owner);
            let value = obj.value.to_runtime_value();
            package_obj.move_to(value, Package::type_tag(), Package::type_layout())?;
            // If the Package is new, we should increase the size of module store
            // But we can not get mutable reference of module store object here
            // So we need to increase the size of module store in publish_module
            new_package = true;
        };
        Ok((package_obj, new_package))
    }

    pub fn tx_context(&self) -> TxContext {
        self.tx_context
            .as_tx_context()
            .expect("Failed to get tx_context")
    }

    pub fn borrow_tx_context(&self) -> PartialVMResult<Value> {
        self.tx_context.borrow_global()
    }

    pub fn add_to_tx_context<T: MoveState>(&mut self, value: T) -> PartialVMResult<()> {
        let mut tx_ctx = self.tx_context.as_tx_context()?;
        tx_ctx
            .add(value)
            .expect("Failed to add value to tx_context");
        self.tx_context = TxContextValue::new(tx_ctx);
        Ok(())
    }

    pub fn timestamp(&self) -> Timestamp {
        let timestamp_id = Timestamp::object_id();
        let timestamp_obj = self
            .get_loaded_object(&timestamp_id)
            .expect("Failed to get timestamp object")
            .expect("Timestamp object must exist");
        let timestamp_ref = timestamp_obj
            .borrow_value(None)
            .expect("Failed to borrow timestamp value");
        let struct_ref = timestamp_ref
            .value_as::<StructRef>()
            .expect("Failed to get struct ref");
        Timestamp::from_runtime_value(
            struct_ref
                .read_ref()
                .expect("Failed to read timestamp value"),
        )
        .expect("Failed to convert timestamp value to timestamp")
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
            let field_key = object_id.field_key();
            let (parent, parent_load_gas) =
                self.load_object(layout_loader, resolver, &parent_id)?;
            let (obj, load_gas) = parent.load_field(layout_loader, resolver, field_key)?;
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
                Some(parent) => {
                    let field_key = object_id.field_key();
                    Ok(parent.get_loaded_field(&field_key))
                }
                None => Ok(None),
            }
        }
    }

    pub fn get_loaded_module(&self, module_id: &ModuleId) -> PartialVMResult<Option<MoveModule>> {
        let module_store_id = ModuleStore::module_store_id();
        let module_store_obj = self
            .get_loaded_object(&module_store_id)?
            .expect("module store object must exist");

        let package_obj =
            module_store_obj.get_loaded_field(&Package::derive_package_key(module_id.address()));
        match package_obj {
            Some(package_obj) => {
                let field_key = FieldKey::derive_module_key(module_id.name());
                let module_field = package_obj.get_loaded_field(&field_key);
                match module_field {
                    Some(field) => field.as_move_module(),
                    None => Ok(None),
                }
            }
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
                let package_key = Package::derive_package_key(module_id.address());
                let (package_obj, _) =
                    module_store_obj.load_field(layout_loader, resolver, package_key)?;
                let field_key = FieldKey::derive_module_key(module_id.name());
                let (module_field, _loaded) =
                    package_obj.load_field(layout_loader, resolver, field_key)?;
                let move_module = module_field.as_move_module()?;
                Ok(move_module.map(|m| m.byte_codes))
            }
            Err(e) => {
                debug!("load_module error: {:?}", e);
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
        // TODO: Publishing module in Rust is only available for genesis transaction.
        // The tx sender will be used ad package object owner,
        // Is the genesis tx sender framework addresses?
        let tx_sender = self.tx_context().sender();
        let (module_store_obj, _) = self.load_object(layout_loader, resolver, &module_store_id)?;
        let (package_obj, new_package) = Self::load_or_create_package_object(
            module_store_obj,
            layout_loader,
            resolver,
            module_id.address(),
            tx_sender,
        )?;
        let module_name = MoveString::from(module_id.name());
        let field_key = FieldKey::derive_module_key(module_id.name());
        let (module_field, _) = package_obj.load_field(layout_loader, resolver, field_key)?;

        let value_type = DynamicField::<MoveString, MoveModule>::type_tag();
        let value_layout = DynamicField::<MoveString, MoveModule>::type_layout();

        let move_module = MoveModule::new(blob);
        let field_value = DynamicField {
            name: module_name,
            value: move_module,
        };
        let runtime_field_value = field_value.to_runtime_value();
        if is_republishing {
            let _old_value = module_field.move_from(Some(&value_type))?;
        }

        module_field.move_to(runtime_field_value, value_type, value_layout)?;

        if !is_republishing {
            // If we directly publish module in Rust, not in Move, we need to increase the size of module store
            package_obj.rt_meta.increase_size()?;
        }
        if new_package {
            // If the Package is new, we should increase the size of module store
            module_store_obj.rt_meta.increase_size()?;
        }
        Ok(())
    }

    pub fn exists_loaded_module(&self, module_id: &ModuleId) -> PartialVMResult<bool> {
        self.get_loaded_module(module_id).map(|m| m.is_some())
    }

    pub fn load_arguments(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StatelessResolver,
        resolved_args: &[ResolvedArg],
    ) -> VMResult<()> {
        for resolved_arg in resolved_args {
            if let ResolvedArg::Object(object_arg) = resolved_arg {
                let object_id = object_arg.object_id();
                let (rt_obj, _) = self
                    .load_object(layout_loader, resolver, object_id)
                    .map_err(|e| e.finish(Location::Module(object::MODULE_ID.clone())))?;
                match object_arg {
                    ObjectArg::Ref(_obj) | ObjectArg::Mutref(_obj) => {
                        let pointer_value = rt_obj
                            .borrow_object(None)
                            .map_err(|e| e.finish(Location::Module(object::MODULE_ID.clone())))?;
                        //We cache the object pointer value in the object_pointer_in_args
                        //Ensure the reference count and the object can not be borrowed in Move
                        self.object_pointer_in_args
                            .insert(object_id.clone(), pointer_value);
                    }
                    ObjectArg::Value(_obj) => {
                        let pointer_value = rt_obj
                            .take_object(None)
                            .map_err(|e| e.finish(Location::Module(object::MODULE_ID.clone())))?;
                        //We cache the object pointer value in the object_pointer_in_args
                        //Ensure the reference count and the object can not be borrowed in Move
                        self.object_pointer_in_args
                            .insert(object_id.clone(), pointer_value);
                    }
                }
            }
        }
        Ok(())
    }

    // into inner
    fn into_inner(self) -> (TxContext, RuntimeObject) {
        let ObjectRuntime {
            tx_context,
            root,
            object_pointer_in_args: _,
        } = self;
        (tx_context.into_inner(), root)
    }

    pub fn into_change_set(self) -> PartialVMResult<(TxContext, StateChangeSet)> {
        let timestamp = self.timestamp();
        let (tx_context, root) = self.into_inner();
        let root_metadata = root.metadata()?.clone();
        let root_change = root
            .into_change(&timestamp)?
            .unwrap_or(ObjectChange::meta(root_metadata.clone()));
        let mut changes = BTreeMap::new();
        for (k, field_change) in root_change.fields {
            changes.insert(k, field_change);
        }
        let change_set = StateChangeSet {
            //TODO should we keep the root ObjectMeta in the change set?
            root_metadata: root_change.metadata,
            changes,
        };
        Ok((tx_context, change_set))
    }
}

pub(crate) fn partial_extension_error(msg: impl ToString) -> PartialVMError {
    log::debug!("PartialVMError: {}", msg.to_string());
    PartialVMError::new(StatusCode::VM_EXTENSION_ERROR).with_message(msg.to_string())
}

pub(crate) fn sum_load_cost(
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
