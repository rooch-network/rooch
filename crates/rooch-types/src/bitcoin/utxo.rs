// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bitcoin::Txid;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::string::MoveString,
    moveos_std::{object::ObjectID, simple_multimap::SimpleMultiMap, tx_context::TxContext},
    state::{MoveState, MoveStructState, MoveStructType},
};
use serde::{Deserialize, Serialize};

use crate::{addresses::BITCOIN_MOVE_ADDRESS, into_address::IntoAddress};

pub const MODULE_NAME: &IdentStr = ident_str!("utxo");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputID {
    /// The txid of the UTXO
    pub txid: AccountAddress,
    /// The vout of the UTXO
    pub vout: u32,
}

impl OutputID {
    pub fn new(txid: AccountAddress, vout: u32) -> Self {
        Self { txid, vout }
    }
}

impl MoveStructType for OutputID {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("OutputID");
}

impl MoveStructState for OutputID {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            AccountAddress::type_layout(),
            u32::type_layout(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXO {
    /// The txid of the UTXO
    pub txid: AccountAddress,
    /// The vout of the UTXO
    pub vout: u32,
    pub value: u64,
    pub seals: SimpleMultiMap<MoveString, ObjectID>,
}

impl MoveStructType for UTXO {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("UTXO");
}

impl MoveStructState for UTXO {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            AccountAddress::type_layout(),
            u32::type_layout(),
            u64::type_layout(),
            SimpleMultiMap::<MoveString, ObjectID>::type_layout(),
        ])
    }
}

/// Rust bindings for BitcoinMove utxo module
pub struct UTXOModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl UTXOModule<'_> {
    pub const EXISTS_UTXO_FUNCTION_NAME: &'static IdentStr = ident_str!("exists_utxo");

    pub fn exists_utxo(&self, txid: Txid, vout: u32) -> Result<bool> {
        let call = Self::create_function_call(
            Self::EXISTS_UTXO_FUNCTION_NAME,
            vec![],
            vec![
                MoveValue::Address(txid.into_address()),
                MoveValue::U32(vout),
            ],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ONE);
        let exists = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<bool>(&value.value).expect("should be a valid Vec<Inscription>")
            })?;
        Ok(exists)
    }
}

impl<'a> ModuleBinding<'a> for UTXOModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;

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
    use super::*;
    use move_core_types::account_address::AccountAddress;
    use moveos_types::moveos_std::object;

    #[test]
    fn test_id() {
        let id = crate::bitcoin::utxo::OutputID::new(
            AccountAddress::from_hex_literal(
                "0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21",
            )
            .unwrap(),
            0,
        );
        let object_id = object::custom_object_id(id, &UTXO::struct_tag());
        //println!("{}", object_id);
        //ensure the object id is same as utxo.move
        assert_eq!(
            object_id,
            ObjectID::from_str(
                "0xb8fc937bf3c15abe49c95fa6906aff29087149f542b48db0cf25dce671a68a63"
            )
            .unwrap()
        );
    }
}
