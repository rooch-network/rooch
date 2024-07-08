// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_core_types::{language_storage::TypeTag, value::MoveTypeLayout};
use move_vm_runtime::native_functions::NativeContext;
use move_vm_types::loaded_data::runtime_types::Type;
use runtime::partial_extension_error;

pub mod resolved_arg;
pub mod runtime;
pub mod runtime_object;
pub mod runtime_object_meta;
pub mod tx_context;

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

/// Asserts that a condition is true, otherwise returns an MoveVM abort with a message.
#[macro_export]
macro_rules! assert_abort {
    ($cond:expr, $abort:expr $(,)?) => {
        if !$cond {
            return Err(move_binary_format::errors::PartialVMError::new(StatusCode::ABORTED)
                .with_sub_status($abort));
        }
    };
    ($cond:expr, $abort:expr, $msg:literal $(,)?) => {
        if !$cond {
            return Err(move_binary_format::errors::PartialVMError::new(StatusCode::ABORTED)
                .with_sub_status($abort)
                .with_message($msg:literal));
        }
    };
    ($cond:expr, $abort:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return Err(move_binary_format::errors::PartialVMError::new(StatusCode::ABORTED)
                .with_sub_status($abort)
                .with_message(format!($fmt, $($arg)*)));
        }
    };
}
