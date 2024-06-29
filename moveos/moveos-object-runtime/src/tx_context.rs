// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_vm_types::values::{GlobalValue, StructRef, Value};
use moveos_types::{moveos_std::tx_context::TxContext, state::MoveState};

/// TxContext in Runtime
pub struct TxContextValue {
    value: GlobalValue,
}

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
