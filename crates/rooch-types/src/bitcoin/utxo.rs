// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::types;
use crate::address::BitcoinAddress;
use crate::addresses::BITCOIN_MOVE_ADDRESS;
use crate::indexer::state::IndexerGlobalState;
use anyhow::Result;
use move_core_types::language_storage::StructTag;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::moveos_std::object;
use moveos_types::state::MoveStructState;
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
pub struct SealPoint {
    pub output_index: u32,
    pub offset: u64,
    pub object_id: ObjectID,
}

impl MoveStructType for SealPoint {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("SealPoint");
}

impl MoveStructState for SealPoint {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            u32::type_layout(),
            u64::type_layout(),
            ObjectID::type_layout(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UTXO {
    /// The txid of the UTXO
    pub txid: AccountAddress,
    /// The vout of the UTXO
    pub vout: u32,
    pub value: u64,
    pub seals: SimpleMultiMap<MoveString, SealPoint>,
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
        seals: SimpleMultiMap<MoveString, SealPoint>,
    ) -> Self {
        Self {
            txid,
            vout,
            value,
            seals,
        }
    }
}

pub fn derive_utxo_id(outpoint: &types::OutPoint) -> ObjectID {
    object::custom_child_object_id(BitcoinUTXOStore::object_id(), outpoint, &UTXO::struct_tag())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UTXOState {
    pub object_id: ObjectID,
    pub owner: AccountAddress,
    pub owner_bitcoin_address: Option<BitcoinAddress>,
    pub flag: u8,
    // There is a case when rooch bitcoin relayer synchronizes the bitcoin node data and processes the `process_block`
    // in the contract, this process takes some time, and the object id in the Global state is deleted, but the ojbect
    // id in the Indexer is not deleted yet. At this time, utxo is empty.
    pub value: Option<UTXO>,
    pub object_type: StructTag,
    pub tx_order: u64,
    pub state_index: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl UTXOState {
    pub fn new_from_global_state(
        state: IndexerGlobalState,
        utxo: Option<UTXO>,
        owner_bitcoin_address: Option<BitcoinAddress>,
    ) -> Self {
        Self {
            object_id: state.object_id,
            owner: state.owner,
            owner_bitcoin_address,
            flag: state.flag,
            value: utxo,
            object_type: state.object_type,
            tx_order: state.tx_order,
            state_index: state.state_index,
            created_at: state.created_at,
            updated_at: state.updated_at,
        }
    }
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
