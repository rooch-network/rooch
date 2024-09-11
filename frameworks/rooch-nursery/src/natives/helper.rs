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



#[derive(Debug, Clone)]
pub struct CommonGasParameters {
    pub load_base: InternalGas,
    pub load_per_byte: InternalGasPerByte,
    pub load_failure: InternalGas,
}

impl CommonGasParameters {
    fn calculate_load_cost(&self, loaded: Option<Option<NumBytes>>) -> InternalGas {
        self.load_base
            + match loaded {
                Some(Some(num_bytes)) => self.load_per_byte * num_bytes,
                Some(None) => self.load_failure,
                None => 0.into(),
            }
    }
}

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
