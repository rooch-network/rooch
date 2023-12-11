// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::FunctionHandleIndex;
use move_binary_format::CompiledModule;
use move_core_types::resolver::MoveResolver;
use move_core_types::vm_status::AbortLocation;
use move_core_types::vm_status::VMStatus;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct MoveAbortExplain {
    pub category_code: u64,
    pub category_name: Option<String>,
    pub reason_code: u64,
    pub reason_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub enum VMStatusExplainView {
    /// The VM status corresponding to an EXECUTED status code
    Executed,
    /// Indicates an error from the VM, e.g. OUT_OF_GAS, INVALID_AUTH_KEY, RET_TYPE_MISMATCH_ERROR
    /// etc.
    /// The code will neither EXECUTED nor ABORTED
    Error(String),

    /// Indicates an `abort` from inside Move code. Contains the location of the abort and the code
    MoveAbort {
        location: AbortLocation,
        reason_code: u64,
    },

    /// Indicates an failure from inside Move code, where the VM could not continue execution, e.g.
    /// dividing by zero or a missing resource
    ExecutionFailure {
        /// status_code in str.
        status_code: String,
        /// status_code in u64.
        status: u64,
        location: AbortLocation,
        function: u16,
        // The function name refers to the function that fails during MoveVM verify or execution,
        // not the function called by the user.
        function_name: Option<String>,
        code_offset: u16,
    },
}

pub fn explain_vm_status<T>(module_resolver: T, vm_status: VMStatus) -> Result<VMStatusExplainView>
where
    T: MoveResolver,
{
    let vm_status_explain = match &vm_status {
        VMStatus::Executed => VMStatusExplainView::Executed,
        VMStatus::Error { status_code, .. } => {
            VMStatusExplainView::Error(format!("{:?}", status_code))
        }
        VMStatus::MoveAbort(location, abort_code) => {
            //TODO find a way to include the description
            //Define a error code description trait, and let caller pass the error description files as argument.
            VMStatusExplainView::MoveAbort {
                location: location.clone(),
                reason_code: *abort_code,
            }
        }
        VMStatus::ExecutionFailure {
            status_code,
            location,
            function,
            code_offset,
            ..
        } => VMStatusExplainView::ExecutionFailure {
            status_code: format!("{:?}", status_code),
            status: (*status_code).into(),
            location: location.clone(),
            function: *function,
            function_name: extract_func_name(location, function, module_resolver),
            code_offset: *code_offset,
        },
    };
    Ok(vm_status_explain)
}

fn extract_func_name<T>(
    location: &AbortLocation,
    function: &u16,
    module_resolver: T,
) -> Option<String>
where
    T: MoveResolver,
{
    match location {
        AbortLocation::Module(module_id) => {
            let module_name = module_id.short_str_lossless();
            let module_bytes = module_resolver.get_module(module_id).ok()?.unwrap();
            let module = CompiledModule::deserialize(&module_bytes).ok()?;
            let func_handle = module.function_handle_at(FunctionHandleIndex::new(*function));
            let func_name = module.identifier_at(func_handle.name).to_string();

            Some(format!("{}::{}", module_name, func_name))
        }
        AbortLocation::Script => None,
    }
}
