// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::vm_status::{StatusCode, VMStatus};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
};
use std::{collections::VecDeque, sync::Arc};

pub const E_NATIVE_FUNCTION_PANIC: u64 = 987654321;

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
            let result = func(&gas_params, context, ty_args, args);
            if cfg!(debug_assertions) {
                match &result {
                    Err(err) => {
                        tracing::debug!("Error in native function: {:?}", err);
                    }
                    Ok(res) => match res {
                        NativeResult::Success {
                            cost: _,
                            ret_vals: _,
                        } => {}
                        NativeResult::Abort { cost, abort_code } => {
                            tracing::debug!(
                                "Abort in native function: cost: {:?}, abort_code: {:?}",
                                cost,
                                abort_code
                            );
                        }
                        NativeResult::OutOfGas { partial_cost } => {
                            tracing::debug!(
                                "OutOfGas in native function: partial_cost: {:?}",
                                partial_cost
                            );
                        }
                    },
                }
            }
            if let Err(err) = &result {
                let status = err.major_status();
                let vm_status = VMStatus::error(status, None);
                let error_message = format!("Native function execution panic {}", err);
                match vm_status.keep_or_discard() {
                    Ok(_) => result,
                    Err(_) => {
                        tracing::error!("{}", error_message);
                        Err(PartialVMError::new(StatusCode::ABORTED)
                            .with_sub_status(E_NATIVE_FUNCTION_PANIC)
                            .with_message(error_message))
                    }
                }
            } else {
                result
            }
        },
    )
}
