// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//Source origin from https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-types/src/base_types.rs

use crate::addresses::MOVEOS_STD_ADDRESS;
use crate::gas_config::GasConfig;
use crate::h256::{self, H256};
use crate::move_std::string::MoveString;
use crate::moveos_std::copyable_any::{Any, AnyTrait};
use crate::moveos_std::object::ObjectID;
use crate::moveos_std::simple_map::SimpleMap;
use crate::state::{MoveState, MoveStructState, MoveStructType};
use anyhow::Result;
use move_core_types::value::{MoveStructLayout, MoveTypeLayout};
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub const TX_CONTEXT_MODULE_NAME: &IdentStr = ident_str!("tx_context");
pub const TX_CONTEXT_STRUCT_NAME: &IdentStr = ident_str!("TxContext");

#[derive(Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct TxContext {
    /// Signer/sender of the transaction
    pub sender: AccountAddress,
    /// Sequence number of this transaction corresponding to sender's account.
    pub sequence_number: u64,
    // The max gas to be used.
    pub max_gas_amount: u64,
    /// Hash of the current transaction
    /// Use the type `Vec<u8>` is to keep consistency with the `TxContext` type in Move
    pub tx_hash: Vec<u8>,
    /// Number of `ObjectID`'s generated during execution of the current transaction
    pub ids_created: u64,
    /// A map for storing context data
    pub map: SimpleMap<MoveString, Any>,
}

impl std::fmt::Debug for TxContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TxContext")
            .field("sender", &self.sender)
            .field("sequence_number", &self.sequence_number)
            .field("max_gas_amount", &self.max_gas_amount)
            .field("tx_hash", &self.tx_hash)
            .field("ids_created", &self.ids_created)
            .field("map", &self.map)
            .finish()
    }
}

impl TxContext {
    pub fn new(
        sender: AccountAddress,
        sequence_number: u64,
        max_gas_amount: u64,
        tx_hash: H256,
    ) -> Self {
        Self {
            sender,
            sequence_number,
            max_gas_amount,
            tx_hash: tx_hash.0.to_vec(),
            ids_created: 0,
            map: SimpleMap::create(),
        }
    }

    /// Create a new TxContext with a zero tx_hash for read-only function call cases
    pub fn new_readonly_ctx(sender: AccountAddress) -> Self {
        //TODO define read-only function gas limit
        Self::new(sender, 0, GasConfig::DEFAULT_MAX_GAS_AMOUNT, H256::zero())
    }

    /// Spawn a new TxContext with a new `ids_created` counter and empty map
    pub fn spawn(self, env: SimpleMap<MoveString, Any>) -> Self {
        Self {
            sender: self.sender,
            sequence_number: self.sequence_number,
            max_gas_amount: self.max_gas_amount,
            tx_hash: self.tx_hash,
            ids_created: 0,
            map: env,
        }
    }

    /// Zero TxContext for some read-only function call cases,
    /// We do not know the sender address and tx_hash in this case
    pub fn zero() -> Self {
        Self {
            sender: AccountAddress::ZERO,
            sequence_number: 0,
            max_gas_amount: GasConfig::DEFAULT_MAX_GAS_AMOUNT,
            tx_hash: vec![0u8; h256::LENGTH],
            ids_created: 0,
            map: SimpleMap::create(),
        }
    }

    /// Derive a globally unique object ID by hashing self.digest | self.ids_created
    pub fn fresh_id(&mut self) -> ObjectID {
        let id = ObjectID::derive_id(self.tx_hash.clone(), self.ids_created);
        self.ids_created += 1;
        id
    }

    /// Return the transaction Hash, to include in new objects
    pub fn tx_hash(&self) -> H256 {
        H256::from_slice(&self.tx_hash)
    }

    pub fn sender(&self) -> AccountAddress {
        self.sender
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        debug_assert!(self.tx_hash.len() == h256::LENGTH);
        bcs::to_bytes(&self).unwrap()
    }

    // for testing
    pub fn random_for_testing_only() -> Self {
        Self::new(
            AccountAddress::random(),
            0,
            GasConfig::DEFAULT_MAX_GAS_AMOUNT,
            H256::random(),
        )
    }

    pub fn add<T: MoveState>(&mut self, value: T) -> Result<()> {
        let type_name = MoveString::from_str(&T::type_tag().to_canonical_string())?;
        let any = Any::pack(value)?;
        self.map.add(type_name, any);
        Ok(())
    }

    pub fn get<T: MoveState>(&self) -> Result<Option<T>> {
        let type_name = MoveString::from_str(&T::type_tag().to_canonical_string())?;
        let any = self.map.borrow(&type_name);
        match any {
            Some(any) => {
                let value = any.clone().unpack::<T>()?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

impl MoveStructType for TxContext {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = TX_CONTEXT_MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = TX_CONTEXT_STRUCT_NAME;
}

impl MoveStructState for TxContext {
    /// Return the layout of the TxContext in Move
    /// TODO: write a macro to auto generate Layout for Rust type.
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![
            MoveTypeLayout::Address,
            MoveTypeLayout::U64,
            MoveTypeLayout::U64,
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::U8)),
            MoveTypeLayout::U64,
            MoveTypeLayout::Struct(SimpleMap::<MoveString, Any>::struct_layout()),
        ])
    }
}

#[cfg(test)]
mod tests {
    use move_core_types::value::MoveValue;

    use super::*;

    #[test]
    pub fn test_tx_context_serialize() {
        let test = TxContext::random_for_testing_only();
        let serialized = test.to_bytes();
        let deserialized: TxContext = bcs::from_bytes(&serialized).unwrap();
        assert_eq!(test, deserialized);
        let decoded_value = MoveValue::simple_deserialize(
            &serialized,
            &(MoveTypeLayout::Struct(TxContext::struct_layout())),
        )
        .unwrap();
        let serialized2 = decoded_value.simple_serialize().unwrap();
        assert_eq!(serialized, serialized2);
    }
}
