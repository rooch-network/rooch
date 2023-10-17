// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::account_address::AccountAddress;
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::values::{Reference, Struct, StructRef};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
};
use moveos_types::moveos_std::object::ObjectID;
use std::{collections::VecDeque, sync::Arc};

// The index of the address field in ObjectID.
pub const OBJECT_ID_HANDLE_FIELD_INDEX: usize = 0;

// The handle type in Move is `&ObjectID`. This function extracts the address from `ObjectID`.
pub fn get_object_id(raw_data: StructRef) -> PartialVMResult<ObjectID> {
    let raw_object_id = raw_data
        .borrow_field(OBJECT_ID_HANDLE_FIELD_INDEX)?
        .value_as::<Reference>()?
        .read_ref()?
        .value_as::<AccountAddress>()?;
    Ok(ObjectID::new(raw_object_id.into()))
}

// The handle type in Move is `ObjectID`. This function extracts the address from `ObjectID`.
pub fn get_object_id_from_struct(raw_data: Struct) -> PartialVMResult<ObjectID> {
    let raw_object_id = raw_data
        .unpack()?
        .next()
        .ok_or_else(|| {
            PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                .with_message("ObjectID must have at least one field".to_string())
        })?
        .value_as::<AccountAddress>()?;
    Ok(ObjectID::new(raw_object_id.into()))
}

pub fn make_module_natives(
    natives: impl IntoIterator<Item = (impl Into<String>, NativeFunction)>,
) -> impl Iterator<Item = (String, NativeFunction)> {
    natives
        .into_iter()
        .map(|(func_name, func)| (func_name.into(), func))
}

pub fn make_native<G>(
    gas_params: G,
    func: impl Fn(&G, &mut NativeContext, Vec<Type>, VecDeque<Value>) -> PartialVMResult<NativeResult>
        + Sync
        + Send
        + 'static,
) -> NativeFunction
where
    G: Send + Sync + 'static,
{
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            func(&gas_params, context, ty_args, args)
        },
    )
}
