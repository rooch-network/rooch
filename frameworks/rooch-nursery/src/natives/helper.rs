
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0


use move_binary_format::errors::{Location, PartialVMError, PartialVMResult};
use move_core_types::{
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    vm_status::StatusCode,
};
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::values::Value;
use moveos_types::{
    moveos_std::object::{Object, ObjectID},
    state::{MoveState, PlaceholderStruct},
};
use std::collections::VecDeque;

 
// =========================================================================================
// Helpers

pub(crate) fn pop_object_id(args: &mut VecDeque<Value>) -> PartialVMResult<ObjectID> {
  let handle = args.pop_back().unwrap();
  ObjectID::from_runtime_value(handle).map_err(|e| {
      if log::log_enabled!(log::Level::Debug) {
          log::warn!("[ObjectRuntime] get_object_id: {:?}", e);
      }
      PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE).with_message(e.to_string())
  })
}
