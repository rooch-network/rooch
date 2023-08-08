// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBundle, MoveFunctionCaller},
    move_option::MoveOption,
    state::MoveState,
    transaction::{FunctionCall, MoveAction},
    tx_context::TxContext,
};
use rooch_types::{
    crypto::BuiltinScheme,
    framework::session_key::{SessionKey, SessionScope},
};

/// Rust bindings for RoochFramework session_key module
pub struct SessionKeyModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> SessionKeyModule<'a> {
    pub const GET_SESSION_KEY_FUNCTION_NAME: &'static IdentStr = ident_str!("get_session_key");
    pub const CREATE_SESSION_KEY_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("create_session_key_entry");

    pub fn get_session_key(
        &self,
        account_address: AccountAddress,
        auth_key: Vec<u8>,
    ) -> Result<Option<SessionKey>> {
        let call = FunctionCall::new(
            Self::function_id(Self::GET_SESSION_KEY_FUNCTION_NAME),
            vec![],
            vec![
                MoveValue::Address(account_address)
                    .simple_serialize()
                    .unwrap(),
                MoveValue::vector_u8(auth_key).simple_serialize().unwrap(),
            ],
        );
        let ctx = TxContext::new_readonly_ctx(account_address);
        let session_key = self.caller.call_function(&ctx, call).map(|mut values| {
            let value = values.pop().expect("should have one return value");
            bcs::from_bytes::<MoveOption<SessionKey>>(&value.value)
                .expect("should be a valid MoveOption<SessionKey>")
                .into()
        })?;
        Ok(session_key)
    }

    pub fn create_session_key_action(
        authentication_key: Vec<u8>,
        scheme: BuiltinScheme,
        scope: SessionScope,
        expiration_time: u64,
        max_inactive_interval: u64,
    ) -> MoveAction {
        Self::create_move_action(
            Self::CREATE_SESSION_KEY_ENTRY_FUNCTION_NAME,
            vec![],
            vec![
                MoveValue::vector_u8(authentication_key),
                MoveValue::U64(scheme.flag() as u64),
                scope.module_address.to_move_value(),
                scope.module_name.to_move_value(),
                scope.function_name.to_move_value(),
                MoveValue::U64(expiration_time),
                MoveValue::U64(max_inactive_interval),
            ],
        )
    }
}

impl<'a> ModuleBundle<'a> for SessionKeyModule<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("session_key");
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
