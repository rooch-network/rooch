// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::addresses::MOVEOS_STD_ADDRESS;
use crate::moveos_std::object;
use crate::moveos_std::object::ObjectID;
use crate::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    moveos_std::tx_context::TxContext,
    state::{MoveStructState, MoveStructType},
    transaction::FunctionCall,
};
use anyhow::Result;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("timestamp");

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Timestamp {
    pub microseconds: u64,
}

impl Timestamp {
    pub fn object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }
}

impl MoveStructType for Timestamp {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("object");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Timestamp");
}

impl MoveStructState for Timestamp {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U64,
        ])
    }
}

/// Rust bindings for MoveosStd timestamp module
pub struct TimestampModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> TimestampModule<'a> {
    pub const NOW_MICROSECONDS_FUNCTION_NAME: &'static IdentStr = ident_str!("now_milliseconds");
    pub const NOW_SECONDS_FUNCTION_NAME: &'static IdentStr = ident_str!("now_seconds");

    pub fn now_milliseconds(&self) -> Result<u64> {
        let call = FunctionCall::new(
            Self::function_id(Self::NOW_MICROSECONDS_FUNCTION_NAME),
            vec![],
            vec![],
        );
        let ctx = TxContext::zero();
        let session_key =
            self.caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    bcs::from_bytes::<u64>(&value.value).expect("should be a valid u64")
                })?;
        Ok(session_key)
    }

    pub fn now_seconds(&self) -> Result<u64> {
        let call = FunctionCall::new(
            Self::function_id(Self::NOW_SECONDS_FUNCTION_NAME),
            vec![],
            vec![],
        );
        let ctx = TxContext::zero();
        let session_key =
            self.caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    bcs::from_bytes::<u64>(&value.value).expect("should be a valid u64")
                })?;
        Ok(session_key)
    }
}

impl<'a> ModuleBinding<'a> for TimestampModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    pub fn test_timestamp_id() {
        let object_id = Timestamp::object_id();
        //println!("{:?}", object_id);
        assert_eq!(
            object_id,
            ObjectID::from_str(
                "0x05921974509dbe44ab84328a625f4a6580a5f89dff3e4e2dec448cb2b1c7f5b9"
            )
            .unwrap()
        );
    }
}
