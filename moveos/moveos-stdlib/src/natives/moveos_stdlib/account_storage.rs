// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use better_any::{Tid, TidAble};
use move_binary_format::errors::PartialVMResult;
use move_core_types::{
    account_address::AccountAddress,
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
};
use serde::{Deserialize, Serialize};
use smallvec::smallvec;
use std::{collections::VecDeque, fmt};

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Module {
    #[serde(with = "serde_bytes")]
    code: Vec<u8>,
}

impl Module {
    pub fn new(code: Vec<u8>) -> Module {
        Module { code }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.code
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Module")
            .field("code", &hex::encode(&self.code))
            .finish()
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModuleBundle {
    codes: Vec<Module>,
}

impl ModuleBundle {
    pub fn new(codes: Vec<Vec<u8>>) -> ModuleBundle {
        ModuleBundle {
            codes: codes.into_iter().map(Module::new).collect(),
        }
    }

    pub fn singleton(code: Vec<u8>) -> ModuleBundle {
        ModuleBundle {
            codes: vec![Module::new(code)],
        }
    }

    pub fn into_inner(self) -> Vec<Vec<u8>> {
        self.codes.into_iter().map(Module::into_inner).collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Module> {
        self.codes.iter()
    }
}

impl fmt::Debug for ModuleBundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModuleBundle")
            .field("codes", &self.codes)
            .finish()
    }
}

impl From<Module> for ModuleBundle {
    fn from(m: Module) -> ModuleBundle {
        ModuleBundle { codes: vec![m] }
    }
}

impl IntoIterator for ModuleBundle {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = Module;

    fn into_iter(self) -> Self::IntoIter {
        self.codes.into_iter()
    }
}

// ========================================================================================
// Module Publishing Logic

/// Abort code when module publishing is requested twice (0x03 == INVALID_STATE)
const EALREADY_REQUESTED: u64 = 0x03_0000;

/// The native code context.
#[derive(Tid, Default)]
pub struct NativeCodeContext {
    /// Remembers whether the publishing of a module bundle was requested during transaction
    /// execution.
    pub requested_module_bundle: Option<PublishRequest>,
}

/// Represents a request for module publishing made from a native call and to be processed by the Moveos VM.
pub struct PublishRequest {
    pub owner: AccountAddress,
    pub bundle: Vec<Vec<u8>>,
}

/***************************************************************************************************
 * native fun request_publish(
 *     owner: address,
 *     bundle: vector<vector<u8>>,
 * )
 *   gas cost: base_cost + unit_cost * bytes_len
 *
 **************************************************************************************************/
#[derive(Clone, Debug)]
pub struct RequestPublishGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

fn native_request_publish(
    gas_params: &RequestPublishGasParameters,
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let mut cost = 0.into();

    let mut bundle = vec![];
    for module in pop_arg!(args, Vec<Value>) {
        let module_bytes = module.value_as::<Vec<u8>>()?;
        cost += gas_params.per_byte * NumBytes::new(module_bytes.len() as u64);
        bundle.push(module_bytes);
    }

    let owner = pop_arg!(args, AccountAddress);

    let code_context = context.extensions_mut().get_mut::<NativeCodeContext>();
    if code_context.requested_module_bundle.is_some() {
        // Can't request second time.
        return Ok(NativeResult::err(cost, EALREADY_REQUESTED));
    }
    code_context.requested_module_bundle = Some(PublishRequest { owner, bundle });
    // TODO(Gas): charge gas for requesting code load (charge for actual code loading done elsewhere)
    Ok(NativeResult::ok(cost, smallvec![]))
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub request_publish: RequestPublishGasParameters,
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [(
        "request_publish",
        make_native(gas_params.request_publish, native_request_publish),
    )];

    make_module_natives(natives)
}
