// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//Source origin from https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-types/src/base_types.rs

use crate::h256::{self, H256};
use crate::object::ObjectID;
use move_core_types::value::{MoveStructLayout, MoveTypeLayout};
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, move_resource::MoveStructType,
};
use serde::{Deserialize, Serialize};

pub const TX_CONTEXT_MODULE_NAME: &IdentStr = ident_str!("tx_context");
pub const TX_CONTEXT_STRUCT_NAME: &IdentStr = ident_str!("TxContext");

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TxContext {
    /// Signer/sender of the transaction
    sender: AccountAddress,
    /// Hash of the current transaction
    tx_hash: Vec<u8>,
    /// Number of `ObjectID`'s generated during execution of the current transaction
    ids_created: u64,
}

impl TxContext {
    pub fn new(sender: AccountAddress, tx_hash: H256) -> Self {
        Self {
            sender,
            tx_hash: tx_hash.0.to_vec(),
            ids_created: 0,
        }
    }

    /// Derive a globally unique object ID by hashing self.digest | self.ids_created
    pub fn fresh_id(&mut self) -> ObjectID {
        let id = ObjectID::derive_id(self.tx_hash.clone(), self.ids_created);
        self.ids_created += 1;
        id
    }

    /// Return the transaction Hash, to include in new objects
    pub fn tx_hash(&self) -> &[u8] {
        &self.tx_hash
    }

    pub fn sender(&self) -> AccountAddress {
        self.sender
    }

    pub fn to_vec(&self) -> Vec<u8> {
        debug_assert!(self.tx_hash.len() == h256::LENGTH);
        bcs::to_bytes(&self).unwrap()
    }

    // for testing
    pub fn random_for_testing_only() -> Self {
        Self::new(AccountAddress::random(), H256::random())
    }

    /// Return the layout of the TxContext in Move
    /// TODO: write a macro to auto generate Layout for Rust type.
    pub fn move_layout() -> MoveTypeLayout {
        MoveTypeLayout::Struct(MoveStructLayout::new(vec![
            MoveTypeLayout::Address,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::U64,
        ]))
    }
}

impl MoveStructType for TxContext {
    const MODULE_NAME: &'static IdentStr = TX_CONTEXT_MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = TX_CONTEXT_STRUCT_NAME;
}

#[cfg(test)]
mod tests {
    use move_core_types::value::MoveValue;

    use super::*;

    #[test]
    pub fn test_tx_context_serialize() {
        let test = TxContext::random_for_testing_only();
        let serialized = test.to_vec();
        let deserialized: TxContext = bcs::from_bytes(&serialized).unwrap();
        assert_eq!(test, deserialized);
        let move_value =
            MoveValue::simple_deserialize(&serialized, &TxContext::move_layout()).unwrap();
        let serialized2 = move_value.simple_serialize().unwrap();
        assert_eq!(serialized, serialized2);
    }
}
