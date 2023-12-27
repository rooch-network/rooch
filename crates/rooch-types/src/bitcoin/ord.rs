// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::types::Transaction;
use crate::addresses::BITCOIN_MOVE_ADDRESS;
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::{option::MoveOption, string::MoveString},
    moveos_std::{
        object::{self, ObjectID},
        tx_context::TxContext,
    },
    state::{MoveState, MoveStructState, MoveStructType},
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("ord");

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
pub struct InscriptionID {
    pub txid: AccountAddress,
    pub index: u32,
}

impl InscriptionID {
    pub fn new(txid: AccountAddress, index: u32) -> Self {
        Self { txid, index }
    }
}

impl MoveStructType for InscriptionID {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("InscriptionID");
}

impl MoveStructState for InscriptionID {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            AccountAddress::type_layout(),
            u32::type_layout(),
        ])
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
pub struct Inscription {
    pub txid: AccountAddress,
    pub index: u32,
    pub body: Vec<u8>,
    pub content_encoding: MoveOption<MoveString>,
    pub content_type: MoveOption<MoveString>,
    pub metadata: Vec<u8>,
    pub metaprotocol: MoveOption<MoveString>,
    pub parent: MoveOption<ObjectID>,
    pub pointer: MoveOption<u64>,
}

impl MoveStructType for Inscription {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Inscription");
}

impl MoveStructState for Inscription {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            AccountAddress::type_layout(),
            u32::type_layout(),
            Vec::<u8>::type_layout(),
            MoveOption::<MoveString>::type_layout(),
            MoveOption::<MoveString>::type_layout(),
            Vec::<u8>::type_layout(),
            MoveOption::<MoveString>::type_layout(),
            MoveOption::<ObjectID>::type_layout(),
            MoveOption::<u64>::type_layout(),
        ])
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, Default)]
pub struct InscriptionRecord {
    pub body: Vec<u8>,
    pub content_encoding: MoveOption<MoveString>,
    pub content_type: MoveOption<MoveString>,
    pub duplicate_field: bool,
    pub incomplete_field: bool,
    pub metadata: Vec<u8>,
    pub metaprotocol: MoveOption<MoveString>,
    pub parent: MoveOption<InscriptionID>,
    pub pointer: MoveOption<u64>,
    pub unrecognized_even_field: bool,
}

impl MoveStructType for InscriptionRecord {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("InscriptionRecord");
}

impl MoveStructState for InscriptionRecord {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            Vec::<u8>::type_layout(),
            MoveOption::<MoveString>::type_layout(),
            MoveOption::<MoveString>::type_layout(),
            bool::type_layout(),
            bool::type_layout(),
            Vec::<u8>::type_layout(),
            MoveOption::<MoveString>::type_layout(),
            MoveOption::<InscriptionID>::type_layout(),
            MoveOption::<u64>::type_layout(),
            bool::type_layout(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InscriptionStore {
    /// The latest transaction index has been processed
    pub latest_tx_index: u64,
    /// The inscriptions table id
    pub inscriptions: ObjectID,
    /// The inscription ids table_vec id
    pub inscription_ids: ObjectID,
}

impl InscriptionStore {
    pub fn object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }
}

impl MoveStructType for InscriptionStore {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("InscriptionStore");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for InscriptionStore {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            u64::type_layout(),
            ObjectID::type_layout(),
            ObjectID::type_layout(),
        ])
    }
}

/// Rust bindings for BitcoinMove ord module
pub struct OrdModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> OrdModule<'a> {
    pub const FROM_TRANSACTION_FUNCTION_NAME: &'static IdentStr =
        ident_str!("from_transaction_bytes");

    pub fn from_transaction(&self, tx: &Transaction) -> Result<Vec<InscriptionRecord>> {
        let call = Self::create_function_call(
            Self::FROM_TRANSACTION_FUNCTION_NAME,
            vec![],
            vec![MoveValue::vector_u8(tx.to_bytes())],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ONE);
        let inscription_key =
            self.caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    bcs::from_bytes::<Vec<InscriptionRecord>>(&value.value)
                        .expect("should be a valid Vec<Inscription>")
                })?;
        Ok(inscription_key)
    }
}

impl<'a> ModuleBinding<'a> for OrdModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
