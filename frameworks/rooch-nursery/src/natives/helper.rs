// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    vm_status::StatusCode,
};
use move_vm_types::values::Value;
use moveos_types::{moveos_std::object::ObjectID, state::MoveState};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct CommonGasParametersOption {
    pub load_base: Option<InternalGas>,
    pub load_per_byte: Option<InternalGasPerByte>,
    pub load_failure: Option<InternalGas>,
}

impl CommonGasParametersOption {
    pub fn zeros() -> Self {
        Self {
            load_base: Some(InternalGas::zero()),
            load_per_byte: Some(InternalGasPerByte::zero()),
            load_failure: Some(InternalGas::zero()),
        }
    }

    pub fn calculate_load_cost(&self, loaded: Option<Option<NumBytes>>) -> InternalGas {
        let load_base = self.load_base.unwrap_or_else(InternalGas::zero);
        let load_per_byte = self.load_per_byte.unwrap_or_else(InternalGasPerByte::zero);
        let load_failure = self.load_failure.unwrap_or_else(InternalGas::zero);

        load_base
            + match loaded {
                Some(Some(num_bytes)) => load_per_byte * num_bytes,
                Some(None) => load_failure,
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
