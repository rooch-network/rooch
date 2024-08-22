// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::types;
use crate::addresses::BITCOIN_MOVE_ADDRESS;
use anyhow::Result;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::moveos_std::object::{self, ObjectMeta};
use moveos_types::state::{MoveStructState, MoveType, ObjectState};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::string::MoveString,
    moveos_std::{object::ObjectID, simple_multimap::SimpleMultiMap, tx_context::TxContext},
    state::{MoveState, MoveStructType},
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("utxo");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinUTXOStore {
    pub next_tx_index: u64,
}

impl BitcoinUTXOStore {
    pub fn object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }

    pub fn genesis_object() -> ObjectState {
        let id = Self::object_id();
        let mut metadata = ObjectMeta::genesis_meta(id, BitcoinUTXOStore::type_tag());
        metadata.to_shared();
        ObjectState::new_with_struct(metadata, Self { next_tx_index: 0 })
            .expect("Create BitcoinUTXOStore Object should success")
    }
}

impl MoveStructType for BitcoinUTXOStore {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BitcoinUTXOStore");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for BitcoinUTXOStore {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![u64::type_layout()])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

impl UTXO {
    pub fn new(
        txid: AccountAddress,
        vout: u32,
        value: u64,
        seals: SimpleMultiMap<MoveString, ObjectID>,
    ) -> Self {
        Self {
            txid,
            vout,
            value,
            seals,
        }
    }

    pub fn object_id(&self) -> ObjectID {
        derive_utxo_id(&types::OutPoint::new(self.txid, self.vout))
    }
}

pub fn derive_utxo_id(outpoint: &types::OutPoint) -> ObjectID {
    object::custom_object_id_with_parent::<types::OutPoint, UTXO>(
        BitcoinUTXOStore::object_id(),
        outpoint,
    )
}

/// Rust bindings for BitcoinMove utxo module
pub struct UTXOModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl UTXOModule<'_> {
    pub const EXISTS_UTXO_FUNCTION_NAME: &'static IdentStr = ident_str!("exists_utxo");

    pub fn exists_utxo(&self, outpoint: &types::OutPoint) -> Result<bool> {
        let call = Self::create_function_call(
            Self::EXISTS_UTXO_FUNCTION_NAME,
            vec![],
            vec![outpoint.to_move_value()],
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
    use super::*;
    use move_core_types::account_address::AccountAddress;

    #[test]
    fn test_id() {
        let outpoint = crate::bitcoin::types::OutPoint::new(
            AccountAddress::from_hex_literal(
                "0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21",
            )
            .unwrap(),
            0,
        );
        let object_id = derive_utxo_id(&outpoint);
        //println!("{}", hex::encode(object_id.to_bytes()));
        //ensure the object id is same as utxo.move
        assert_eq!(
            object_id,
            ObjectID::from_bytes(
                hex::decode("02826a5e56581ba5ab84c39976f27cf3578cf524308b4ffc123922dfff507e514db8fc937bf3c15abe49c95fa6906aff29087149f542b48db0cf25dce671a68a63").unwrap()
            )
            .unwrap()
        );
    }
}
