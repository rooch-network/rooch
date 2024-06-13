// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::types::{OutPoint, Transaction};
use crate::address::{BitcoinAddress, RoochAddress};
use crate::addresses::BITCOIN_MOVE_ADDRESS;
use crate::indexer::state::IndexerObjectState;
use crate::into_address::IntoAddress;
use anyhow::Result;
use move_core_types::language_storage::{StructTag, TypeTag};
use move_core_types::value::MoveTypeLayout;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::state::{MoveState, MoveStructState, MoveStructType};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::{option::MoveOption, string::MoveString},
    moveos_std::{
        object::{self, ObjectID},
        tx_context::TxContext,
    },
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

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
    pub offset: u64,
    pub sequence_number: u32,
    pub inscription_number: u32,
    pub is_curse: bool,
    pub body: Vec<u8>,
    pub content_encoding: MoveOption<MoveString>,
    pub content_type: MoveOption<MoveString>,
    pub metadata: Vec<u8>,
    pub metaprotocol: MoveOption<MoveString>,
    pub parents: Vec<ObjectID>,
    pub pointer: MoveOption<u64>,
    pub rune: MoveOption<u128>,
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
            u64::type_layout(),
            u32::type_layout(),
            u32::type_layout(),
            bool::type_layout(),
            Vec::<u8>::type_layout(),
            MoveOption::<MoveString>::type_layout(),
            MoveOption::<MoveString>::type_layout(),
            Vec::<u8>::type_layout(),
            MoveOption::<MoveString>::type_layout(),
            Vec::<ObjectID>::type_layout(),
            MoveOption::<u64>::type_layout(),
            MoveOption::<u128>::type_layout(),
        ])
    }
}

pub fn derive_inscription_id(inscription_id: &InscriptionID) -> ObjectID {
    object::custom_child_object_id(
        InscriptionStore::object_id(),
        inscription_id,
        &Inscription::struct_tag(),
    )
}

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct Envelope<T> {
    pub input: u32,
    pub offset: u32,
    pub pushnum: bool,
    pub stutter: bool,
    pub payload: T,
}

impl<T> MoveStructType for Envelope<T>
where
    T: MoveStructType + DeserializeOwned,
{
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("Envelope");

    fn type_params() -> Vec<TypeTag> {
        vec![TypeTag::Struct(Box::new(T::struct_tag()))]
    }

    fn struct_tag() -> StructTag {
        StructTag {
            address: Self::ADDRESS,
            module: Self::MODULE_NAME.to_owned(),
            name: Self::STRUCT_NAME.to_owned(),
            type_params: vec![T::struct_tag().into()],
        }
    }
}

impl<T> MoveStructState for Envelope<T>
where
    T: MoveStructState,
{
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            u32::type_layout(),
            u32::type_layout(),
            bool::type_layout(),
            bool::type_layout(),
            MoveTypeLayout::Struct(T::struct_layout()),
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
    pub parents: Vec<InscriptionID>,
    pub pointer: MoveOption<u64>,
    pub unrecognized_even_field: bool,
    pub rune: Option<u128>,
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
            Vec::<InscriptionID>::type_layout(),
            MoveOption::<u64>::type_layout(),
            bool::type_layout(),
            MoveOption::<u128>::type_layout(),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InscriptionStore {
    /// The inscriptions ids table_vec object id
    pub inscriptions: ObjectID,
    /// cursed inscription number generator
    pub cursed_inscription_count: u32,
    /// blessed inscription number generator
    pub blessed_inscription_count: u32,
    /// sequence number generator
    pub next_sequence_number: u32,
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
            ObjectID::type_layout(),
            u32::type_layout(),
            u32::type_layout(),
            u32::type_layout(),
        ])
    }
}

#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub struct SatPoint {
    pub outpoint: OutPoint,
    pub offset: u64,
}

impl Display for SatPoint {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.outpoint, self.offset)
    }
}

/// Rust bindings for BitcoinMove ord module
pub struct OrdModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> OrdModule<'a> {
    pub const FROM_TRANSACTION_FUNCTION_NAME: &'static IdentStr =
        ident_str!("from_transaction_bytes");

    pub fn from_transaction(&self, tx: &Transaction) -> Result<Vec<Inscription>> {
        let call = Self::create_function_call(
            Self::FROM_TRANSACTION_FUNCTION_NAME,
            vec![],
            vec![MoveValue::vector_u8(tx.to_bytes())],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ONE);
        let inscriptions =
            self.caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    bcs::from_bytes::<Vec<Inscription>>(&value.value)
                        .expect("should be a valid Vec<Inscription>")
                })?;
        Ok(inscriptions)
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InscriptionState {
    pub object_id: ObjectID,
    pub owner: RoochAddress,
    pub owner_bitcoin_address: Option<BitcoinAddress>,
    pub flag: u8,
    pub value: Inscription,
    pub object_type: StructTag,
    pub tx_order: u64,
    pub state_index: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl InscriptionState {
    pub fn new_from_object_state(
        state: IndexerObjectState,
        inscription: Inscription,
        owner_bitcoin_address: Option<BitcoinAddress>,
    ) -> Self {
        Self {
            object_id: state.object_id,
            owner: state.owner,
            owner_bitcoin_address,
            flag: state.flag,
            value: inscription,
            object_type: state.object_type,
            tx_order: state.tx_order,
            state_index: state.state_index,
            created_at: state.created_at,
            updated_at: state.updated_at,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
pub struct BitcoinInscriptionID {
    pub txid: bitcoin::Txid,
    pub index: u32,
}

impl BitcoinInscriptionID {
    pub fn new(txid: bitcoin::Txid, index: u32) -> Self {
        Self { txid, index }
    }
}

impl From<BitcoinInscriptionID> for InscriptionID {
    fn from(inscription: BitcoinInscriptionID) -> Self {
        InscriptionID {
            txid: inscription.txid.into_address(),
            index: inscription.index,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Curse {
    DuplicateField,
    IncompleteField,
    NotAtOffsetZero,
    NotInFirstInput,
    Pointer,
    Pushnum,
    Reinscription,
    Stutter,
    UnrecognizedEvenField,
}
