// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_return_value::FunctionResult,
    move_types::FunctionId,
    transaction::{FunctionCall, MoveAction},
    tx_context::TxContext,
};
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
    value::MoveValue,
};

pub trait MoveFunctionCaller: Send + Sync {
    fn call_function(&self, ctx: &TxContext, call: FunctionCall) -> Result<FunctionResult>;

    fn as_module_binding<'a, M: ModuleBinding<'a>>(&'a self) -> M
    where
        Self: Sized,
    {
        M::new(self)
    }
}

impl<C> MoveFunctionCaller for &C
where
    C: MoveFunctionCaller,
{
    fn call_function(&self, ctx: &TxContext, call: FunctionCall) -> Result<FunctionResult> {
        (*self).call_function(ctx, call)
    }
}

pub trait ModuleBinding<'a> {
    const MODULE_NAME: &'static IdentStr;
    const MODULE_ADDRESS: AccountAddress;

    fn module_id() -> ModuleId {
        ModuleId::new(Self::MODULE_ADDRESS, Self::MODULE_NAME.to_owned())
    }

    fn function_id(function_name: &IdentStr) -> FunctionId {
        FunctionId::new(Self::module_id(), function_name.to_owned())
    }

    /// Construct a MoveAction for a function call
    fn create_move_action(
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<MoveValue>,
    ) -> MoveAction {
        MoveAction::Function(Self::create_function_call(function_name, ty_args, args))
    }

    /// Çonstruct a FunctionCall
    fn create_function_call(
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<MoveValue>,
    ) -> FunctionCall {
        FunctionCall::new(
            Self::function_id(function_name),
            ty_args,
            args.into_iter()
                .map(|v| v.simple_serialize().expect("Failed to serialize MoveValue"))
                .collect(),
        )
    }

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized;
}
