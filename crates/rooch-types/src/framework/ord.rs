// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::bitcoin_types::Transaction;
use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::option::MoveOption,
    moveos_std::{
        object::{self, ObjectID},
        tx_context::TxContext,
    },
    state::{MoveState, MoveStructState, MoveStructType},
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("ord");

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, Default)]
pub struct InscriptionRecord {
    pub body: MoveOption<Vec<u8>>,
    pub content_encoding: MoveOption<Vec<u8>>,
    pub content_type: MoveOption<Vec<u8>>,
    pub duplicate_field: bool,
    pub incomplete_field: bool,
    pub metadata: MoveOption<Vec<u8>>,
    pub metaprotocol: MoveOption<Vec<u8>>,
    pub parent: MoveOption<Vec<u8>>,
    pub pointer: MoveOption<Vec<u8>>,
    pub unrecognized_even_field: bool,
}

impl MoveStructType for InscriptionRecord {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("InscriptionRecord");
}

impl MoveStructState for InscriptionRecord {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            MoveOption::<Vec<u8>>::type_layout(),
            MoveOption::<Vec<u8>>::type_layout(),
            MoveOption::<Vec<u8>>::type_layout(),
            bool::type_layout(),
            bool::type_layout(),
            MoveOption::<Vec<u8>>::type_layout(),
            MoveOption::<Vec<u8>>::type_layout(),
            MoveOption::<Vec<u8>>::type_layout(),
            MoveOption::<Vec<u8>>::type_layout(),
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
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
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

/// Rust bindings for RoochFramework ord module
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
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
