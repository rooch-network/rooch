// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::types::{OutPoint, Transaction};
use crate::addresses::BITCOIN_MOVE_ADDRESS;
use crate::into_address::{FromAddress, IntoAddress};
use anyhow::{bail, Result};
use bitcoin::constants::MAX_SCRIPT_ELEMENT_SIZE;
use bitcoin::{opcodes, script};
use move_core_types::language_storage::{StructTag, TypeTag};
use move_core_types::value::MoveTypeLayout;
use move_core_types::{account_address::AccountAddress, ident_str, identifier::IdentStr};
use moveos_types::moveos_std::object::ObjectMeta;
use moveos_types::state::{MoveState, MoveStructState, MoveStructType, MoveType, ObjectState};
use moveos_types::{
    h256::H256,
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
use tag::Tag;

pub const MODULE_NAME: &IdentStr = ident_str!("ord");
pub const PROTOCOL_ID: [u8; 3] = *b"ord";
pub const BODY_TAG: [u8; 0] = [];

pub mod tag;

#[derive(PartialEq, Clone, Copy, Hash, Eq, PartialOrd, Ord)]
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
    pub fn new<TID: IntoAddress>(txid: TID, index: u32) -> Self {
        Self {
            txid: txid.into_address(),
            index,
        }
    }

    pub fn object_id(&self) -> ObjectID {
        derive_inscription_id(self)
    }
}

impl FromStr for InscriptionID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        const TXID_LEN: usize = 64;
        const MIN_LEN: usize = TXID_LEN + 2;
        if s.len() < MIN_LEN {
            bail!(
                "Invalid InscriptionID length: {}",
                format!("{}, len: {} < {}", s, s.len(), MIN_LEN)
            );
        }

        let txid = bitcoin::Txid::from_str(&s[..TXID_LEN])?;
        let separator = s.chars().nth(TXID_LEN).unwrap();

        if separator != 'i' {
            bail!("Invalid InscriptionID separator: {}", separator);
        }
        let index = &s[TXID_LEN + 1..];
        let index = index.parse()?;
        Ok(InscriptionID {
            txid: txid.into_address(),
            index,
        })
    }
}

impl Display for InscriptionID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}i{}",
            bitcoin::Txid::from_address(self.txid),
            self.index
        )
    }
}

impl Debug for InscriptionID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}i{}",
            bitcoin::Txid::from_address(self.txid),
            self.index
        )
    }
}

impl Serialize for InscriptionID {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            #[derive(Serialize)]
            struct Value {
                txid: AccountAddress,
                index: u32,
            }
            Value {
                txid: self.txid,
                index: self.index,
            }
            .serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for InscriptionID {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Self::from_str(&s).map_err(serde::de::Error::custom)
        } else {
            #[derive(Deserialize)]
            struct Value {
                txid: AccountAddress,
                index: u32,
            }
            let value = Value::deserialize(deserializer)?;
            Ok(InscriptionID {
                txid: value.txid,
                index: value.index,
            })
        }
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
    pub id: InscriptionID,
    pub location: SatPoint,
    pub sequence_number: u32,
    pub inscription_number: u32,
    pub is_cursed: bool,
    pub charms: u16,
    pub body: Vec<u8>,
    pub content_encoding: MoveOption<MoveString>,
    pub content_type: MoveOption<MoveString>,
    pub metadata: Vec<u8>,
    pub metaprotocol: MoveOption<MoveString>,
    pub parents: Vec<InscriptionID>,
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
            InscriptionID::type_layout(),
            SatPoint::type_layout(),
            u32::type_layout(),
            u32::type_layout(),
            bool::type_layout(),
            u16::type_layout(),
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
        self.id
    }

    pub fn object_id(&self) -> ObjectID {
        derive_inscription_id(&self.id())
    }

    pub fn inscription_number(&self) -> i32 {
        if self.is_cursed {
            -(self.inscription_number as i32)
        } else {
            self.inscription_number as i32
        }
    }

    pub fn metaprotocol(&self) -> Option<&str> {
        self.metaprotocol.as_ref().map(|s| s.as_str())
    }

    pub fn metadata(&self) -> &[u8] {
        &self.metadata
    }

    pub fn parents(&self) -> &[InscriptionID] {
        &self.parents
    }

    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_ref().map(|s| s.as_str())
    }

    pub fn content_encoding(&self) -> Option<&str> {
        self.content_encoding.as_ref().map(|s| s.as_str())
    }

    pub fn pointer(&self) -> Option<u64> {
        self.pointer.as_ref().map(|p| *p)
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn set_metaprotocol(&mut self, metaprotocol: String) {
        self.metaprotocol = Some(metaprotocol.into()).into();
    }

    pub fn set_metadata(&mut self, metadata: Vec<u8>) {
        self.metadata = metadata;
    }

    pub fn set_parents(&mut self, parents: Vec<InscriptionID>) {
        self.parents = parents;
    }

    pub fn set_content_type(&mut self, content_type: String) {
        self.content_type = Some(content_type.into()).into();
    }

    pub fn set_content_encoding(&mut self, content_encoding: String) {
        self.content_encoding = Some(content_encoding.into()).into();
    }

    pub fn set_pointer(&mut self, pointer: u64) {
        self.pointer = Some(pointer).into();
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
    }

    pub fn set_rune(&mut self, rune: u128) {
        self.rune = Some(rune).into();
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

#[derive(PartialEq, Clone, Serialize, Deserialize, Eq, Default)]
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

impl InscriptionRecord {
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_ref().map(|s| s.as_str())
    }

    pub fn content_encoding(&self) -> Option<&str> {
        self.content_encoding.as_ref().map(|s| s.as_str())
    }

    pub fn metadata(&self) -> &[u8] {
        &self.metadata
    }

    pub fn metaprotocol(&self) -> Option<&str> {
        self.metaprotocol.as_ref().map(|s| s.as_str())
    }

    pub fn parents(&self) -> &[InscriptionID] {
        &self.parents
    }

    pub fn pointer(&self) -> Option<u64> {
        self.pointer.as_ref().map(|p| *p)
    }

    pub fn rune(&self) -> Option<u128> {
        self.rune
    }

    pub fn set_content_type(&mut self, content_type: String) {
        self.content_type = Some(content_type.into()).into();
    }

    pub fn set_content_encoding(&mut self, content_encoding: String) {
        self.content_encoding = Some(content_encoding.into()).into();
    }

    pub fn set_metadata(&mut self, metadata: Vec<u8>) {
        self.metadata = metadata;
    }

    pub fn set_metaprotocol(&mut self, metaprotocol: String) {
        self.metaprotocol = Some(metaprotocol.into()).into();
    }

    pub fn set_parents(&mut self, parents: Vec<InscriptionID>) {
        self.parents = parents;
    }

    pub fn set_pointer(&mut self, pointer: u64) {
        self.pointer = Some(pointer).into();
    }

    pub fn set_rune(&mut self, rune: u128) {
        self.rune = Some(rune);
    }

    pub fn append_reveal_script_to_builder(&self, mut builder: script::Builder) -> script::Builder {
        builder = builder
            .push_opcode(opcodes::OP_FALSE)
            .push_opcode(opcodes::all::OP_IF)
            .push_slice(PROTOCOL_ID);

        Tag::ContentType.append(
            &mut builder,
            &self
                .content_type
                .as_ref()
                .map(|content_type| content_type.as_str().as_bytes().to_vec()),
        );
        Tag::ContentEncoding.append(
            &mut builder,
            &self
                .content_encoding
                .as_ref()
                .map(|content_encoding| content_encoding.as_str().as_bytes().to_vec()),
        );
        Tag::Metaprotocol.append(
            &mut builder,
            &self
                .metaprotocol
                .as_ref()
                .map(|metaprotocol| metaprotocol.as_str().as_bytes().to_vec()),
        );
        Tag::Parent.append_array(
            &mut builder,
            &self
                .parents
                .iter()
                .map(|parent| {
                    let mut buffer = parent.txid.to_vec();
                    buffer.extend_from_slice(&parent.index.to_le_bytes());
                    buffer
                })
                .collect(),
        );
        //TODO should support delegate
        //Tag::Delegate.append(&mut builder, &self.delegate);
        Tag::Pointer.append(
            &mut builder,
            &self
                .pointer
                .as_ref()
                .map(|pointer| pointer.to_le_bytes().to_vec()),
        );
        let metadata = if self.metadata.is_empty() {
            None
        } else {
            Some(self.metadata.clone())
        };
        Tag::Metadata.append(&mut builder, &metadata);
        Tag::Rune.append(
            &mut builder,
            &self.rune.as_ref().map(|rune| rune.to_le_bytes().to_vec()),
        );

        if !self.body.is_empty() {
            builder = builder.push_slice(BODY_TAG);
            for chunk in self.body.chunks(MAX_SCRIPT_ELEMENT_SIZE) {
                builder = builder.push_slice::<&script::PushBytes>(chunk.try_into().unwrap());
            }
        }

        builder.push_opcode(opcodes::all::OP_ENDIF)
    }
}

impl Debug for InscriptionRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InscriptionRecord")
            .field("body(len)", &self.body.len())
            .field("content_encoding", &self.content_encoding)
            .field("content_type", &self.content_type)
            .field("duplicate_field", &self.duplicate_field)
            .field("incomplete_field", &self.incomplete_field)
            .field("metadata", &self.metadata)
            .field("metaprotocol", &self.metaprotocol)
            .field("parents", &self.parents)
            .field("pointer", &self.pointer)
            .field("unrecognized_even_field", &self.unrecognized_even_field)
            .field("rune", &self.rune)
            .finish()
    }
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
    pub unbound_inscription_count: u32,
    pub lost_sats: u64,
    /// sequence number generator
    pub next_sequence_number: u32,
}

impl InscriptionStore {
    pub fn object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }

    pub fn genesis_with_state_root(
        state_root: H256,
        size: u64,
        store: InscriptionStore,
    ) -> ObjectState {
        let id = Self::object_id();
        let mut metadata = ObjectMeta::genesis_meta(id, Self::type_tag());
        metadata.state_root = Some(state_root);
        metadata.size = size;
        metadata.to_shared();
        ObjectState::new_with_struct(metadata, store)
            .expect("Create InscriptionStore Object should success")
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
            u64::type_layout(),
            u32::type_layout(),
        ])
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct SatPoint {
    pub outpoint: OutPoint,
    pub offset: u64,
}

impl SatPoint {
    pub fn outpoint(&self) -> bitcoin::OutPoint {
        self.outpoint.clone().into()
    }
}

impl MoveStructType for SatPoint {
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("SatPoint");
}

impl MoveStructState for SatPoint {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            OutPoint::type_layout(),
            u64::type_layout(),
        ])
    }
}

impl FromStr for SatPoint {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts = s.split(':');
        let txid = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing txid"))?;
        let vout = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing vout"))?;
        let offset = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing offset"))?;
        let txid = bitcoin::Txid::from_str(txid)?;
        let vout = u32::from_str(vout)?;
        let offset = u64::from_str(offset)?;
        Ok(SatPoint {
            outpoint: OutPoint {
                txid: txid.into_address(),
                vout,
            },
            offset,
        })
    }
}

impl Display for SatPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txid = bitcoin::Txid::from_address(self.outpoint.txid);
        write!(f, "{}:{}:{}", txid, self.outpoint.vout, self.offset)
    }
}

impl Serialize for SatPoint {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(&self)
        } else {
            #[derive(Serialize)]
            struct Value {
                outpoint: OutPoint,
                offset: u64,
            }
            Value {
                outpoint: self.outpoint.clone(),
                offset: self.offset,
            }
            .serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for SatPoint {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Self::from_str(&s).map_err(serde::de::Error::custom)
        } else {
            #[derive(Deserialize)]
            struct Value {
                outpoint: OutPoint,
                offset: u64,
            }
            let value = Value::deserialize(deserializer)?;
            Ok(SatPoint {
                outpoint: value.outpoint,
                offset: value.offset,
            })
        }
    }
}

/// Rust bindings for BitcoinMove ord module
pub struct OrdModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> OrdModule<'a> {
    pub const PARSE_INSCRIPTION_FROM_TX_FUNCTION_NAME: &'static IdentStr =
        ident_str!("parse_inscription_from_tx");
    pub const MATCH_UTXO_AND_GENERATE_SAT_POINT_FUNCTION_NAME: &'static IdentStr =
        ident_str!("match_utxo_and_generate_sat_point");

    pub fn parse_inscription_from_tx(
        &self,
        tx: &Transaction,
    ) -> Result<Vec<Envelope<InscriptionRecord>>> {
        let call = Self::create_function_call(
            Self::PARSE_INSCRIPTION_FROM_TX_FUNCTION_NAME,
            vec![],
            vec![tx.to_move_value()],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ONE);
        let inscriptions =
            self.caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    bcs::from_bytes::<Vec<Envelope<InscriptionRecord>>>(&value.value)
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
