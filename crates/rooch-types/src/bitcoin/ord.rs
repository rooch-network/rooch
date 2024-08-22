// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::types::Transaction;
use crate::addresses::BITCOIN_MOVE_ADDRESS;
use crate::into_address::{FromAddress, IntoAddress};
use anyhow::{bail, Result};
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
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display};
use std::str::FromStr;

pub const MODULE_NAME: &IdentStr = ident_str!("ord");

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize, Hash, Eq)]
pub struct InscriptionID {
    pub txid: AccountAddress,
    pub index: u32,
}

impl Default for InscriptionID {
    fn default() -> Self {
        Self {
            txid: AccountAddress::ZERO,
            index: 0,
        }
    }
}

impl InscriptionID {
    pub fn new(txid: AccountAddress, index: u32) -> Self {
        Self { txid, index }
    }

    pub fn object_id(&self) -> ObjectID {
        derive_inscription_id(self)
    }
}

impl FromStr for InscriptionID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        BitcoinInscriptionID::from_str(s).map(Into::into)
    }
}

impl Debug for InscriptionID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InscriptionID")
            .field("txid(move)", &self.txid)
            .field(
                "txid(bitcoin)",
                &bitcoin::Txid::from_address(self.txid).to_string(),
            )
            .field("index", &self.index)
            .field("id", &format!("{}", self))
            .finish()
    }
}

impl Display for InscriptionID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bitcoin_inscription_id: BitcoinInscriptionID = (*self).into();
        write!(f, "{}", bitcoin_inscription_id)
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

impl Inscription {
    pub fn id(&self) -> InscriptionID {
        InscriptionID {
            txid: self.txid,
            index: self.index,
        }
    }

    pub fn object_id(&self) -> ObjectID {
        derive_inscription_id(&self.id())
    }
}

pub fn derive_inscription_id(inscription_id: &InscriptionID) -> ObjectID {
    object::custom_object_id_with_parent::<InscriptionID, Inscription>(
        InscriptionStore::object_id(),
        inscription_id,
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
            u32::type_layout(),
            u32::type_layout(),
            u32::type_layout(),
        ])
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
pub struct SatPoint {
    pub output_index: u32,
    pub offset: u64,
    pub object_id: ObjectID,
}

impl MoveStructType for SatPoint {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("SatPoint");
}

impl MoveStructState for SatPoint {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            u32::type_layout(),
            u64::type_layout(),
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
    pub const MATCH_UTXO_AND_GENERATE_SAT_POINT_FUNCTION_NAME: &'static IdentStr =
        ident_str!("match_utxo_and_generate_sat_point");

    pub fn from_transaction(
        &self,
        tx: &Transaction,
        input_utxo_values: Vec<u64>,
        next_inscription_number: u32,
        next_sequence_number: u32,
    ) -> Result<Vec<Inscription>> {
        let call = Self::create_function_call(
            Self::FROM_TRANSACTION_FUNCTION_NAME,
            vec![],
            vec![
                MoveValue::vector_u8(tx.to_bytes()),
                input_utxo_values.to_move_value(),
                next_inscription_number.to_move_value(),
                next_sequence_number.to_move_value(),
            ],
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

    pub fn match_utxo_and_generate_sat_point(
        &self,
        offset: u64,
        seal_object_id: ObjectID,
        tx: &Transaction,
        input_utxo_values: Vec<u64>,
        input_index: u64,
    ) -> Result<(bool, SatPoint)> {
        let call = Self::create_function_call(
            Self::MATCH_UTXO_AND_GENERATE_SAT_POINT_FUNCTION_NAME,
            vec![],
            vec![
                offset.to_move_value(),
                seal_object_id.to_move_value(),
                tx.to_move_value(),
                input_utxo_values.to_move_value(),
                input_index.to_move_value(),
            ],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ONE);
        let result = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let sat_point_value = values.pop().expect("should have return values");
                let bool_value = values.pop().expect("should have return values");
                let sat_point = bcs::from_bytes::<SatPoint>(&sat_point_value.value)
                    .expect("should be a valid SatPoint");
                let is_match =
                    bcs::from_bytes::<bool>(&bool_value.value).expect("should be a valid bool");
                (is_match, sat_point)
            })?;
        Ok(result)
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

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
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

impl From<InscriptionID> for BitcoinInscriptionID {
    fn from(inscription: InscriptionID) -> Self {
        BitcoinInscriptionID {
            txid: bitcoin::Txid::from_address(inscription.txid),
            index: inscription.index,
        }
    }
}

impl Display for BitcoinInscriptionID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}i{}", self.txid, self.index)
    }
}

impl FromStr for BitcoinInscriptionID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        const TXID_LEN: usize = 64;
        const MIN_LEN: usize = TXID_LEN + 2;
        if s.len() < MIN_LEN {
            bail!(
                "Invalid BitcoinInscriptionID length: {}",
                format!("{}, len: {} < {}", s, s.len(), MIN_LEN)
            );
        }

        let txid = bitcoin::Txid::from_str(&s[..TXID_LEN])?;
        let separator = s.chars().nth(TXID_LEN).unwrap();

        if separator != 'i' {
            bail!("Invalid BitcoinInscriptionID separator: {}", separator);
        }
        let index = &s[TXID_LEN + 1..];
        let index = index.parse()?;
        Ok(BitcoinInscriptionID { txid, index })
    }
}

impl<'de> Deserialize<'de> for BitcoinInscriptionID {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::from_str(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl Serialize for BitcoinInscriptionID {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
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
