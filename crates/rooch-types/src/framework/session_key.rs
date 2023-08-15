// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{addresses::ROOCH_FRAMEWORK_ADDRESS, crypto::BuiltinScheme};
use anyhow::Result;
use move_core_types::value::MoveValue;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::{
    module_binding::{ModuleBundle, MoveFunctionCaller},
    move_option::MoveOption,
    state::MoveState,
    transaction::{FunctionCall, MoveAction},
    tx_context::TxContext,
};
use moveos_types::{
    move_string::MoveAsciiString,
    state::{MoveStructState, MoveStructType},
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SessionScope {
    pub module_address: AccountAddress,
    pub module_name: MoveAsciiString,
    pub function_name: MoveAsciiString,
}

impl SessionScope {
    pub fn new(module_address: AccountAddress, module_name: &str, function_name: &str) -> Self {
        Self {
            module_address,
            module_name: MoveAsciiString::from_str(module_name).expect("invalid module name"),
            function_name: MoveAsciiString::from_str(function_name).expect("invalid function name"),
        }
    }
}

impl MoveStructType for SessionScope {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("session_key");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SessionScope");
}

impl MoveStructState for SessionScope {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Address,
            <MoveAsciiString as MoveStructState>::type_layout(),
            <MoveAsciiString as MoveStructState>::type_layout(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKey {
    pub authentication_key: Vec<u8>,
    pub scheme: u64,
    pub scopes: Vec<SessionScope>,
    pub expiration_time: u64,
    pub last_active_time: u64,
    pub max_inactive_interval: u64,
}

impl MoveStructType for SessionKey {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("session_key");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SessionKey");
}

impl MoveStructState for SessionKey {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                <SessionScope as MoveStructState>::type_layout(),
            )),
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::U64,
        ])
    }
}

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
