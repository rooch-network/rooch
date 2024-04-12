// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use crate::authentication_key::AuthenticationKey;
use anyhow::{Error, Result};
use move_core_types::value::MoveValue;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::ascii::MoveAsciiString,
    move_std::option::MoveOption,
    moveos_std::tx_context::TxContext,
    serde::Readable,
    state::{MoveState, MoveStructState, MoveStructType},
    transaction::{FunctionCall, MoveAction},
};
use serde::{Deserialize, Serialize};
use serde_with::hex::Hex;
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SessionScope {
    pub module_address: AccountAddress,
    pub module_name: MoveAsciiString,
    pub function_name: MoveAsciiString,
}

impl SessionScope {
    pub fn new(
        module_address: AccountAddress,
        module_name: &str,
        function_name: &str,
    ) -> Result<Self> {
        let module_name_value = match MoveAsciiString::from_str(module_name) {
            Ok(v) => v,
            Err(_) => return Err(Error::msg("invalid module name")),
        };
        let function_name_value = match MoveAsciiString::from_str(function_name) {
            Ok(v) => v,
            Err(_) => return Err(Error::msg("invalid function name")),
        };
        Ok(Self {
            module_address,
            module_name: module_name_value,
            function_name: function_name_value,
        })
    }

    fn is_asterisk(s: &MoveAsciiString) -> bool {
        s.as_bytes() == b"*"
    }

    pub fn check_scope_match(&self, function: &FunctionCall) -> bool {
        if &self.module_address != function.function_id.module_id.address() {
            return false;
        }
        if !Self::is_asterisk(&self.module_name)
            && self.module_name.to_string().as_str()
                != function.function_id.module_id.name().as_str()
        {
            return false;
        }
        if !Self::is_asterisk(&self.function_name)
            && self.function_name.to_string().as_str()
                != function.function_id.function_name.as_str()
        {
            return false;
        }
        true
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
            MoveAsciiString::type_layout(),
            MoveAsciiString::type_layout(),
        ])
    }
}

impl Display for SessionScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}::{}::{}",
            self.module_address, self.module_name, self.function_name
        )
    }
}

impl FromStr for SessionScope {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split("::");
        let module_address = AccountAddress::from_str(
            parts
                .next()
                .ok_or(anyhow::anyhow!("invalid session scope"))?,
        )?;
        let module_name = parts
            .next()
            .ok_or(anyhow::anyhow!("invalid session scope"))?;
        let function_name = parts
            .next()
            .ok_or(anyhow::anyhow!("invalid session scope"))?;
        Self::new(module_address, module_name, function_name)
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKey {
    #[serde_as(as = "Readable<Hex, _>")]
    pub authentication_key: Vec<u8>,
    #[serde_as(as = "Vec<Readable<DisplayFromStr, _>>")]
    pub scopes: Vec<SessionScope>,
    pub create_time: u64,
    pub last_active_time: u64,
    pub max_inactive_interval: u64,
}

impl SessionKey {
    pub fn authentication_key(&self) -> AuthenticationKey {
        AuthenticationKey::new(self.authentication_key.clone())
    }
    pub fn is_expired(&self) -> bool {
        if self.max_inactive_interval == 0 {
            return false;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("should get system time")
            .as_secs();
        now - self.last_active_time > self.max_inactive_interval
    }

    pub fn is_scope_match(&self, function: &FunctionCall) -> bool {
        self.scopes
            .iter()
            .any(|scope| scope.check_scope_match(function))
    }

    pub fn is_scope_match_with_action(&self, action: &MoveAction) -> bool {
        match action {
            MoveAction::Script(_) => false,
            MoveAction::ModuleBundle(_) => false,
            MoveAction::Function(function) => self.is_scope_match(function),
        }
    }
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
            move_core_types::value::MoveTypeLayout::Vector(Box::new(SessionScope::type_layout())),
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
        auth_key: &AuthenticationKey,
    ) -> Result<Option<SessionKey>> {
        let call = FunctionCall::new(
            Self::function_id(Self::GET_SESSION_KEY_FUNCTION_NAME),
            vec![],
            vec![
                MoveValue::Address(account_address)
                    .simple_serialize()
                    .unwrap(),
                MoveValue::vector_u8(auth_key.as_ref().to_vec())
                    .simple_serialize()
                    .unwrap(),
            ],
        );
        let ctx = TxContext::new_readonly_ctx(account_address);
        let session_key =
            self.caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    bcs::from_bytes::<MoveOption<SessionKey>>(&value.value)
                        .expect("should be a valid MoveOption<SessionKey>")
                        .into()
                })?;
        Ok(session_key)
    }

    pub fn create_session_key_action(
        authentication_key: Vec<u8>,
        scope: SessionScope,
        max_inactive_interval: u64,
    ) -> MoveAction {
        Self::create_move_action(
            Self::CREATE_SESSION_KEY_ENTRY_FUNCTION_NAME,
            vec![],
            vec![
                MoveValue::vector_u8(authentication_key),
                scope.module_address.to_move_value(),
                scope.module_name.to_move_value(),
                scope.function_name.to_move_value(),
                MoveValue::U64(max_inactive_interval),
            ],
        )
    }
}

impl<'a> ModuleBinding<'a> for SessionKeyModule<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("session_key");
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::SessionScope;
    use move_core_types::{account_address::AccountAddress, ident_str, language_storage::ModuleId};
    use moveos_types::{
        move_std::ascii::MoveAsciiString, move_types::FunctionId, transaction::FunctionCall,
    };

    fn do_test_scope_match(scope: &SessionScope, function: &FunctionCall, expect: bool) {
        assert_eq!(
            scope.check_scope_match(function),
            expect,
            "scope: {:?}, function: {:?}",
            scope,
            function
        );
    }

    #[test]
    fn test_scope_match() {
        let session_scope = SessionScope {
            module_address: AccountAddress::ONE,
            module_name: MoveAsciiString::from_str("test").unwrap(),
            function_name: MoveAsciiString::from_str("test").unwrap(),
        };
        let cases = vec![
            (
                FunctionCall::new(
                    FunctionId::new(
                        ModuleId::new(AccountAddress::ONE, ident_str!("test").to_owned()),
                        ident_str!("test").to_owned(),
                    ),
                    vec![],
                    vec![],
                ),
                true,
            ),
            (
                FunctionCall::new(
                    FunctionId::new(
                        ModuleId::new(AccountAddress::TWO, ident_str!("test").to_owned()),
                        ident_str!("test").to_owned(),
                    ),
                    vec![],
                    vec![],
                ),
                false,
            ),
            (
                FunctionCall::new(
                    FunctionId::new(
                        ModuleId::new(AccountAddress::ONE, ident_str!("test2").to_owned()),
                        ident_str!("test").to_owned(),
                    ),
                    vec![],
                    vec![],
                ),
                false,
            ),
            (
                FunctionCall::new(
                    FunctionId::new(
                        ModuleId::new(AccountAddress::ONE, ident_str!("test").to_owned()),
                        ident_str!("test2").to_owned(),
                    ),
                    vec![],
                    vec![],
                ),
                false,
            ),
        ];
        cases
            .into_iter()
            .for_each(|c| do_test_scope_match(&session_scope, &c.0, c.1));
    }

    #[test]
    fn test_check_scope_match_asterisk() {
        let session_scope = SessionScope {
            module_address: AccountAddress::ONE,
            module_name: MoveAsciiString::from_str("*").unwrap(),
            function_name: MoveAsciiString::from_str("*").unwrap(),
        };
        let cases = vec![
            (
                FunctionCall::new(
                    FunctionId::new(
                        ModuleId::new(AccountAddress::ONE, ident_str!("test").to_owned()),
                        ident_str!("test").to_owned(),
                    ),
                    vec![],
                    vec![],
                ),
                true,
            ),
            (
                FunctionCall::new(
                    FunctionId::new(
                        ModuleId::new(AccountAddress::TWO, ident_str!("test").to_owned()),
                        ident_str!("test").to_owned(),
                    ),
                    vec![],
                    vec![],
                ),
                false,
            ),
            (
                FunctionCall::new(
                    FunctionId::new(
                        ModuleId::new(AccountAddress::ONE, ident_str!("test2").to_owned()),
                        ident_str!("test").to_owned(),
                    ),
                    vec![],
                    vec![],
                ),
                true,
            ),
            (
                FunctionCall::new(
                    FunctionId::new(
                        ModuleId::new(AccountAddress::ONE, ident_str!("test").to_owned()),
                        ident_str!("test2").to_owned(),
                    ),
                    vec![],
                    vec![],
                ),
                true,
            ),
        ];
        cases
            .into_iter()
            .for_each(|c| do_test_scope_match(&session_scope, &c.0, c.1));
    }
}
