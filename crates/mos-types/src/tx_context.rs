// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//Source origin from https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-types/src/base_types.rs

use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, move_resource::MoveStructType,
};
use serde::{Deserialize, Serialize};
use smt::HashValue;

use crate::object::ObjectID;

pub const TX_CONTEXT_MODULE_NAME: &IdentStr = ident_str!("tx_context");
pub const TX_CONTEXT_STRUCT_NAME: &IdentStr = ident_str!("TxContext");

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TxContext {
    /// Signer/sender of the transaction
    sender: AccountAddress,
    /// Digest of the current transaction
    tx_hash: HashValue,
    /// Number of `ObjectID`'s generated during execution of the current transaction
    ids_created: u64,
}

impl TxContext {
    pub fn new(sender: AccountAddress, tx_hash: HashValue) -> Self {
        Self {
            sender,
            tx_hash,
            ids_created: 0,
        }
    }

    /// Derive a globally unique object ID by hashing self.digest | self.ids_created
    pub fn fresh_id(&mut self) -> ObjectID {
        let id = ObjectID::derive_id(self.tx_hash.to_vec(), self.ids_created);
        self.ids_created += 1;
        id
    }

    /// Return the transaction Hash, to include in new objects
    pub fn tx_hash(&self) -> HashValue {
        self.tx_hash
    }

    pub fn sender(&self) -> AccountAddress {
        self.sender
    }

    pub fn to_vec(&self) -> Vec<u8> {
        bcs::to_bytes(&self).unwrap()
    }

    // for testing
    pub fn random_for_testing_only() -> Self {
        Self::new(AccountAddress::random(), HashValue::random())
    }
}

impl MoveStructType for TxContext {
    const MODULE_NAME: &'static IdentStr = TX_CONTEXT_MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = TX_CONTEXT_STRUCT_NAME;
}
