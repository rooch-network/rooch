// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBundle, MoveFunctionCaller},
    move_option::MoveOption,
    state::{MoveStructState, MoveStructType},
    transaction::FunctionCall,
    tx_context::TxContext,
};
use serde::{Deserialize, Serialize};

/// Rust bindings for RoochFramework account_authencation::AuthenticationKey
#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct AuthenticationKey<V> {
    pub authencation_key: Vec<u8>,
    auth_validator: std::marker::PhantomData<V>,
}

impl<V> MoveStructType for AuthenticationKey<V>
where
    V: MoveStructType,
{
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = AuthenticationKeyModule::MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("AuthenticationKey");

    fn struct_tag() -> move_core_types::language_storage::StructTag {
        move_core_types::language_storage::StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![V::type_tag()],
        }
    }
}

impl<V> MoveStructState for AuthenticationKey<V>
where
    V: MoveStructState,
{
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
        ])
    }
}

/// Rust bindings for RoochFramework account_authencation module
pub struct AuthenticationKeyModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> AuthenticationKeyModule<'a> {
    const GET_AUTHENTICATION_KEY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("get_authentication_key");

    pub fn get_authentication_key<V: MoveStructType>(
        &self,
        address: AccountAddress,
    ) -> Result<Option<Vec<u8>>> {
        let call = FunctionCall::new(
            Self::function_id(Self::GET_AUTHENTICATION_KEY_FUNCTION_NAME),
            vec![V::type_tag()],
            vec![MoveValue::Address(address)
                .simple_serialize()
                .expect("address should serialize")],
        );
        let ctx = TxContext::new_readonly_ctx(address);
        self.caller.call_function(&ctx, call).map(|values| {
            let value = values.get(0).expect("Expected return value");
            let result =
                MoveOption::<Vec<u8>>::from_bytes(&value.value).expect("Expected Option<address>");
            result.into()
        })
    }
}

impl<'a> ModuleBundle<'a> for AuthenticationKeyModule<'a> {
    const MODULE_NAME: &'static IdentStr = ident_str!("account_authencation");
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
