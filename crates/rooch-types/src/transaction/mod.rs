// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use accumulator::accumulator_info::AccumulatorInfo;
use anyhow::Result;
use framework_types::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::vm_status::KeptVMStatus;
use moveos_types::state::{MoveState, MoveStructState, MoveStructType};
use moveos_types::transaction::TransactionExecutionInfo;
use moveos_types::{h256::H256, transaction::TransactionOutput};
use serde::{Deserialize, Deserializer, Serialize};

pub mod authenticator;
mod ledger_transaction;
pub mod rooch;

use crate::indexer::transaction::IndexerTransaction;
pub use authenticator::Authenticator;
pub use ledger_transaction::{
    L1Block, L1BlockWithBody, L1Transaction, LedgerTransaction, LedgerTxData,
};
pub use rooch::{RoochTransaction, RoochTransactionData};

pub const TRANSACTION_SEQUENCE_INFO_STR: &str = "TransactionSequenceInfo";

pub const TRANSACTION_SEQUENCE_INFO_FIELDS: &[&str] = &[
    "tx_order",
    "tx_order_signature",
    "tx_accumulator_root",
    "tx_timestamp",
    "tx_accumulator_frozen_subtree_roots",
    "tx_accumulator_num_leaves",
    "tx_accumulator_num_nodes",
];

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RawTransaction {
    pub raw: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AuthenticatorInfo {
    pub chain_id: u64,
    pub authenticator: Authenticator,
}

impl AuthenticatorInfo {
    pub fn new(chain_id: u64, authenticator: Authenticator) -> Self {
        Self {
            chain_id,
            authenticator,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("encode authenticator info should success")
    }
}

impl From<AuthenticatorInfo> for Vec<u8> {
    fn from(info: AuthenticatorInfo) -> Self {
        info.to_bytes()
    }
}

///`TransactionSequenceInfo` represents the result of sequence a transaction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TransactionSequenceInfo {
    /// The tx order
    pub tx_order: u64,
    /// The tx order signature, it is the signature of the sequencer to commit the tx order.
    pub tx_order_signature: Vec<u8>,
    /// The tx accumulator root after the tx is append to the accumulator.
    pub tx_accumulator_root: H256,
    /// The tx accumulator info after the tx is append to the accumulator.
    // pub tx_accumulator_info: Option<AccumulatorInfo>,
    /// The timestamp of the sequencer when the tx is sequenced, in millisecond.
    pub tx_timestamp: u64,

    /// Frozen subtree roots of the accumulator.
    #[serde(default)]
    pub tx_accumulator_frozen_subtree_roots: Vec<H256>,
    /// The total number of leaves in the accumulator.
    #[serde(default)]
    pub tx_accumulator_num_leaves: u64,
    /// The total number of nodes in the accumulator.
    #[serde(default)]
    pub tx_accumulator_num_nodes: u64,
}

impl TransactionSequenceInfo {
    pub fn new(
        tx_order: u64,
        tx_order_signature: Vec<u8>,
        tx_accumulator_info: AccumulatorInfo,
        tx_timestamp: u64,
    ) -> TransactionSequenceInfo {
        TransactionSequenceInfo {
            tx_order,
            tx_order_signature,
            tx_accumulator_root: tx_accumulator_info.accumulator_root,
            tx_timestamp,
            tx_accumulator_frozen_subtree_roots: tx_accumulator_info.frozen_subtree_roots,
            tx_accumulator_num_leaves: tx_accumulator_info.num_leaves,
            tx_accumulator_num_nodes: tx_accumulator_info.num_nodes,
        }
    }

    pub fn tx_accumulator_info(&self) -> AccumulatorInfo {
        AccumulatorInfo::new(
            self.tx_accumulator_root,
            self.tx_accumulator_frozen_subtree_roots.clone(),
            self.tx_accumulator_num_leaves,
            self.tx_accumulator_num_nodes,
        )
    }
}

impl MoveStructType for TransactionSequenceInfo {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("transaction");
    const STRUCT_NAME: &'static IdentStr = ident_str!("TransactionSequenceInfoV2");
}

impl MoveStructState for TransactionSequenceInfo {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
            move_core_types::value::MoveTypeLayout::Vector(Box::new(
                move_core_types::value::MoveTypeLayout::U8,
            )),
            move_core_types::value::MoveTypeLayout::U64,
            Vec::<Vec<u8>>::type_layout(),
            move_core_types::value::MoveTypeLayout::U64,
            move_core_types::value::MoveTypeLayout::U64,
        ])
    }
}

// #[derive(Clone, Debug, Serialize, Deserialize)]
// struct TransactionSequenceInfoWrapper {
//     tx_order: u64,
//     tx_order_signature: Vec<u8>,
//     tx_accumulator_root: H256,
//     tx_timestamp: u64,
//
//     #[serde(default)]
//     tx_accumulator_info: Option<AccumulatorInfo>,
// }
//
// impl Serialize for TransactionSequenceInfo {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let tx_accumulator_info = AccumulatorInfo::new(
//             self.tx_accumulator_root,
//             self.tx_accumulator_frozen_subtree_roots.clone(),
//             self.tx_accumulator_num_leaves,
//             self.tx_accumulator_num_nodes,
//         );
//         let wrapper = TransactionSequenceInfoWrapper {
//             tx_order: self.tx_order,
//             tx_order_signature: self.tx_order_signature.clone(),
//             tx_accumulator_root: self.tx_accumulator_root,
//             tx_timestamp: self.tx_timestamp,
//             tx_accumulator_info: Some(tx_accumulator_info),
//         };
//
//         wrapper.serialize(serializer)
//     }
// }
//
// impl<'de> Deserialize<'de> for TransactionSequenceInfo {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         println!("[Debug] Deserialize deserialize start");
//         let wrapper = TransactionSequenceInfoWrapper::deserialize(deserializer)?;
//         println!("[Debug] Deserialize deserialize 01");
//         let tx_accumulator_info = wrapper.tx_accumulator_info.unwrap_or_default();
//         Ok(TransactionSequenceInfo {
//             tx_order: wrapper.tx_order,
//             tx_order_signature: wrapper.tx_order_signature,
//             tx_accumulator_root: wrapper.tx_accumulator_root,
//             tx_timestamp: wrapper.tx_timestamp,
//             tx_accumulator_frozen_subtree_roots: tx_accumulator_info.frozen_subtree_roots,
//             tx_accumulator_num_leaves: tx_accumulator_info.num_leaves,
//             tx_accumulator_num_nodes: tx_accumulator_info.num_nodes,
//         })
//     }
// }

// Implement custom Deserialize for TransactionSequenceInfo
impl<'de> Deserialize<'de> for TransactionSequenceInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TransactionSequenceInfoVisitor;

        impl<'de> serde::de::Visitor<'de> for TransactionSequenceInfoVisitor {
            type Value = TransactionSequenceInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Expect TransactionSequenceInfo for deserializer")
            }

            // To be compatible with old data, tx_accumulator_frozen_subtree_roots, tx_accumulator_num_leaves,
            // and tx_accumulator_num_nodes are allowed to be missing.
            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: serde::de::SeqAccess<'de>,
            {
                // let size_hint = seq.size_hint();
                // println!(
                //     "[DEBUG] TransactionSequenceInfo Deserializer size_hint {:?} ",
                //     size_hint
                // );
                let tx_order = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_order field when deserialize TransactionSequenceInfo"))?;
                let tx_order_signature = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_order_signature field when deserialize TransactionSequenceInfo"))?;
                let tx_accumulator_root = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_accumulator_root field when deserialize TransactionSequenceInfo"))?;
                let tx_timestamp: u64 = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_timestamp field when deserialize TransactionSequenceInfo"))?;

                // Ignore deserialize error "unexpected end of input" for old data when missing field
                let tx_accumulator_frozen_subtree_roots = seq
                    .next_element()
                    .unwrap_or_else(|e| None)
                    .unwrap_or(vec![]);
                // Ignore deserialize error "unexpected end of input" for old data when missing field
                let tx_accumulator_num_leaves =
                    seq.next_element().unwrap_or_else(|e| None).unwrap_or(0u64);
                // Ignore deserialize error "unexpected end of input" for old data when missing field
                let tx_accumulator_num_nodes =
                    seq.next_element().unwrap_or_else(|e| None).unwrap_or(0u64);

                Ok(TransactionSequenceInfo {
                    tx_order,
                    tx_order_signature,
                    tx_accumulator_root,
                    tx_timestamp,
                    tx_accumulator_frozen_subtree_roots,
                    tx_accumulator_num_leaves,
                    tx_accumulator_num_nodes,
                })
            }
        }

        // deserializer.deserialize_seq(TransactionSequenceInfoVisitor)
        deserializer.deserialize_struct(
            TRANSACTION_SEQUENCE_INFO_STR,
            TRANSACTION_SEQUENCE_INFO_FIELDS,
            TransactionSequenceInfoVisitor,
        )

        // deserializer.deserialize_byte_buf()
        // deserializer.deserialize_newtype_struct(
        //     TRANSACTION_SEQUENCE_INFO_STR,
        //     TransactionSequenceInfoVisitor,
        // )

        // let v: Vec<u8> = Deserialize::deserialize(deserializer)?;
    }
}

// // Implement custom Deserialize for TransactionSequenceInfo
// impl<'de> Deserialize<'de> for TransactionSequenceInfo {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         struct TransactionSequenceInfoVisitor;
//
//         impl<'de> serde::de::Visitor<'de> for TransactionSequenceInfoVisitor {
//             type Value = TransactionSequenceInfo;
//
//             fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//                 formatter.write_str("Expect TransactionSequenceInfo")
//             }
//
//             // To be compatible with old data, tx_accumulator_frozen_subtree_roots, tx_accumulator_num_leaves,
//             // and tx_accumulator_num_nodes are allowed to be missing.
//             fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
//             where
//                 S: serde::de::SeqAccess<'de>,
//             {
//                 let size_hint = seq.size_hint();
//                 println!(
//                     "[DEBUG] TransactionSequenceInfo Deserializer size_hint {:?} ",
//                     size_hint
//                 );
//                 let tx_order = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_order field when deserialize TransactionSequenceInfo"))?;
//                 println!(
//                     "[DEBUG] TransactionSequenceInfo Deserializer tx_order {:?} ",
//                     tx_order
//                 );
//                 let tx_order_signature = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_order_signature field when deserialize TransactionSequenceInfo"))?;
//                 let tx_accumulator_root = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_accumulator_root field when deserialize TransactionSequenceInfo"))?;
//                 let tx_timestamp: u64 = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_timestamp field when deserialize TransactionSequenceInfo"))?;
//                 let tx_accumulator_frozen_subtree_roots = seq.next_element()?.unwrap_or(vec![]);
//                 println!(
//                     "[DEBUG] TransactionSequenceInfo Deserializer tx_accumulator_frozen_subtree_roots {:?} ",
//                     tx_accumulator_frozen_subtree_roots
//                 );
//                 let tx_accumulator_num_leaves = seq.next_element()?.unwrap_or(0u64);
//                 let tx_accumulator_num_nodes = seq.next_element()?.unwrap_or(0u64);
//
//                 Ok(TransactionSequenceInfo {
//                     tx_order,
//                     tx_order_signature,
//                     tx_accumulator_root,
//                     tx_timestamp,
//                     tx_accumulator_frozen_subtree_roots,
//                     tx_accumulator_num_leaves,
//                     tx_accumulator_num_nodes,
//                 })
//             }
//         }
//
//         deserializer.deserialize_any(TransactionSequenceInfoVisitor)
//     }
// }

// // Implement custom Deserialize for TransactionSequenceInfo
// impl<'de> Deserialize<'de> for TransactionSequenceInfo {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         // enum Field {
//         //     TxOrder,
//         //     TxOrderSignature,
//         //     TxAccumulatorRoot,
//         //     TxTimestamp,
//         //     TxAccumulatorFrozenSubtreeRoots,
//         //     TxAccumulatorNumLeaves,
//         //     TxAccumulatorNumNodes,
//         // }
//         //
//         // impl<'de> Deserialize<'de> for Field {
//         //     fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
//         //     where
//         //         D: Deserializer<'de>,
//         //     {
//         //         struct FieldVisitor;
//         //
//         //         impl<'de> Visitor<'de> for FieldVisitor {
//         //             type Value = Field;
//         //
//         //             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         //                 formatter.write_str("Invalid field`")
//         //             }
//         //
//         //             fn visit_str<E>(self, value: &str) -> Result<Field, E>
//         //             where
//         //                 E: serde::de::Error,
//         //             {
//         //                 match value {
//         //                     "tx_order" => Ok(Field::TxOrder),
//         //                     "tx_order_signature" => Ok(Field::TxOrderSignature),
//         //                     "tx_accumulator_root" => Ok(Field::TxAccumulatorRoot),
//         //                     "tx_timestamp" => Ok(Field::TxTimestamp),
//         //                     "tx_accumulator_frozen_subtree_roots" => {
//         //                         Ok(Field::TxAccumulatorFrozenSubtreeRoots)
//         //                     }
//         //                     "tx_accumulator_num_leaves" => Ok(Field::TxAccumulatorNumLeaves),
//         //                     "tx_accumulator_num_nodes" => Ok(Field::TxAccumulatorNumNodes),
//         //                     _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
//         //                 }
//         //             }
//         //         }
//         //
//         //         deserializer.deserialize_identifier(FieldVisitor)
//         //     }
//         // }
//
//         struct TransactionSequenceInfoVisitor;
//
//         impl<'de> Visitor<'de> for TransactionSequenceInfoVisitor {
//             type Value = TransactionSequenceInfo;
//
//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("Expect TransactionSequenceInfo Struct")
//             }
//
//             fn visit_seq<V>(self, mut seq: V) -> Result<TransactionSequenceInfo, V::Error>
//             where
//                 V: SeqAccess<'de>,
//             {
//                 // let secs = seq
//                 //     .next_element()?
//                 //     .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
//                 // let nanos = seq
//                 //     .next_element()?
//                 //     .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
//                 // Ok(TransactionSequenceInfo::new(secs, nanos))
//
//                 let size_hint = seq.size_hint();
//                 println!(
//                     "[DEBUG] TransactionSequenceInfo Deserializer size_hint {:?} ",
//                     size_hint
//                 );
//                 let tx_order: u64 = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_order field when deserialize TransactionSequenceInfo"))?;
//
//                 println!("[Debug] visit_seq tx_order {:?}", tx_order);
//                 let tx_order_signature: Vec<u8> = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_order_signature field when deserialize TransactionSequenceInfo"))?;
//                 // println!(
//                 //     "[Debug] visit_seq tx_order_signature {:?}",
//                 //     tx_order_signature
//                 // );
//                 // let tx_accumulator_root_bytes: Vec<u8> = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_accumulator_root field when deserialize TransactionSequenceInfo"))?;
//                 let tx_accumulator_root: H256 = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_accumulator_root field when deserialize TransactionSequenceInfo"))?;
//                 let tx_timestamp: u64 = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_timestamp field when deserialize TransactionSequenceInfo"))?;
//                 println!("[Debug] visit_seq tx_timestamp {:?}", tx_timestamp);
//                 println!(
//                     "[Debug] visit_seq size_hint  {}",
//                     seq.size_hint().unwrap_or(0)
//                 );
//
//                 println!("[Debug] visit_seq tx_accumulator_frozen_subtree_roots 00",);
//                 let tx_accumulator_frozen_subtree_roots =
//                     seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_accumulator_root field when deserialize TransactionSequenceInfo"))?;
//                 println!(
//                     "[Debug] visit_seq tx_accumulator_frozen_subtree_roots 01 {:?}",
//                     tx_accumulator_frozen_subtree_roots
//                 );
//
//                 // let tx_accumulator_frozen_subtree_roots = if seq.size_hint().unwrap_or(0) > 0 {
//                 // let tx_accumulator_frozen_subtree_roots_opt = seq.next_element()?;
//                 // println!(
//                 //     "[Debug] visit_seq tx_accumulator_frozen_subtree_roots_opt {:?}",
//                 //     tx_accumulator_frozen_subtree_roots_opt
//                 // );
//                 // let tx_accumulator_frozen_subtree_roots_bytes: Vec<u8> =
//                 //     tx_accumulator_frozen_subtree_roots_opt.unwrap_or(vec![]);
//                 // println!(
//                 //     "[Debug] visit_seq tx_accumulator_frozen_subtree_roots_bytes 0000 {:?}",
//                 //     tx_accumulator_frozen_subtree_roots_bytes
//                 // );
//
//                 // println!("[Debug] visit_seq tx_accumulator_frozen_subtree_roots 00",);
//                 // let tx_accumulator_frozen_subtree_roots =
//                 //     seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_accumulator_root field when deserialize TransactionSequenceInfo"))?;
//                 // println!(
//                 //     "[Debug] visit_seq tx_accumulator_frozen_subtree_roots 01 {:?}",
//                 //     tx_accumulator_frozen_subtree_roots
//                 // );
//                 //
//                 // tx_accumulator_frozen_subtree_roots
//                 // vec![H256::zero()]
//                 // } else {
//                 //     println!("[Debug] visit_seq tx_accumulator_frozen_subtree_roots 02",);
//                 //     vec![]
//                 // };
//                 let tx_accumulator_num_leaves = if seq.size_hint().unwrap_or(0) > 0 {
//                     seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_accumulator_num_leaves field when deserialize TransactionSequenceInfo"))?
//                 } else {
//                     0u64
//                 };
//                 let tx_accumulator_num_nodes = if seq.size_hint().unwrap_or(0) > 0 {
//                     seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Missing or invalid tx_accumulator_num_nodes field when deserialize TransactionSequenceInfo"))?
//                 } else {
//                     0u64
//                 };
//                 // let tx_accumulator_num_leaves: u64 = seq.next_element()?.unwrap_or(0u64);
//                 // let tx_accumulator_num_nodes: u64 = seq.next_element()?.unwrap_or(0u64);
//
//                 // let tx_accumulator_root = H256::from_slice(tx_accumulator_root_bytes.as_slice());
//                 // let tx_accumulator_root = H256::from_str(tx_accumulator_root_bytes.into())?;
//                 // let tx_accumulator_frozen_subtree_roots = tx_accumulator_frozen_subtree_roots_bytes
//                 //     .into_iter()
//                 //     .map(|v| H256::from_str(v.into()).map_err(Into::into))
//                 //     .collect::<Result<Vec<_>>>()?;
//                 // let tx_accumulator_frozen_subtree_roots =
//                 //     tx_accumulator_frozen_subtree_roots_bytes.clone();
//                 Ok(TransactionSequenceInfo {
//                     tx_order,
//                     tx_order_signature,
//                     tx_accumulator_root,
//                     tx_timestamp,
//                     tx_accumulator_frozen_subtree_roots,
//                     tx_accumulator_num_leaves,
//                     tx_accumulator_num_nodes,
//                 })
//             }
//
//             // fn visit_map<V>(self, mut map: V) -> Result<TransactionSequenceInfo, V::Error>
//             // where
//             //     V: MapAccess<'de>,
//             // {
//             //     let mut tx_order = None;
//             //     let mut tx_order_signature = None;
//             //     let mut tx_accumulator_root = None;
//             //     let mut tx_timestamp = None;
//             //     let mut tx_accumulator_frozen_subtree_roots = None;
//             //     let mut tx_accumulator_num_leaves = None;
//             //     let mut tx_accumulator_num_nodes = None;
//             //
//             //     while let Some(key) = map.next_key()? {
//             //         match key {
//             //             Field::TxOrder => {
//             //                 if tx_order.is_some() {
//             //                     return Err(serde::de::Error::duplicate_field("tx_order"));
//             //                 }
//             //                 tx_order = Some(map.next_value()?);
//             //                 println!("[Debug] visit_map tx_order {:?}", tx_order);
//             //             }
//             //             Field::TxOrderSignature => {
//             //                 if tx_order_signature.is_some() {
//             //                     return Err(serde::de::Error::duplicate_field(
//             //                         "tx_order_signature",
//             //                     ));
//             //                 }
//             //                 tx_order_signature = Some(map.next_value()?);
//             //             }
//             //             Field::TxAccumulatorRoot => {
//             //                 if tx_accumulator_root.is_some() {
//             //                     return Err(serde::de::Error::duplicate_field(
//             //                         "tx_accumulator_root",
//             //                     ));
//             //                 }
//             //                 tx_accumulator_root = Some(map.next_value()?);
//             //             }
//             //             Field::TxTimestamp => {
//             //                 if tx_timestamp.is_some() {
//             //                     return Err(serde::de::Error::duplicate_field("tx_timestamp"));
//             //                 }
//             //                 tx_timestamp = Some(map.next_value()?);
//             //
//             //                 println!("[Debug] visit_map tx_timestamp {:?}", tx_timestamp);
//             //             }
//             //             Field::TxAccumulatorFrozenSubtreeRoots => {
//             //                 if tx_accumulator_frozen_subtree_roots.is_some() {
//             //                     return Err(serde::de::Error::duplicate_field(
//             //                         "tx_accumulator_frozen_subtree_roots",
//             //                     ));
//             //                 }
//             //
//             //                 // tx_accumulator_frozen_subtree_roots =
//             //                 //     map.next_entry()?.map(|(k, v)| v);
//             //                 if map.size_hint().unwrap_or(0) > 0 {
//             //                     tx_accumulator_frozen_subtree_roots = Some(map.next_value()?);
//             //                     println!(
//             //                         "[Debug] visit_map tx_accumulator_frozen_subtree_roots {:?}",
//             //                         tx_accumulator_frozen_subtree_roots
//             //                     );
//             //                 }
//             //                 // _tx_accumulator_frozen_subtree_roots = Some(vec![]);
//             //             }
//             //             Field::TxAccumulatorNumLeaves => {
//             //                 if tx_accumulator_num_leaves.is_some() {
//             //                     return Err(serde::de::Error::duplicate_field(
//             //                         "tx_accumulator_num_leaves",
//             //                     ));
//             //                 }
//             //                 // tx_accumulator_num_leaves = map.next_entry()?.map(|(_k, v)| v);
//             //                 if map.size_hint().unwrap_or(0) > 0 {
//             //                     tx_accumulator_num_leaves = Some(map.next_value()?);
//             //                 }
//             //             }
//             //             Field::TxAccumulatorNumNodes => {
//             //                 if tx_accumulator_num_nodes.is_some() {
//             //                     return Err(serde::de::Error::duplicate_field(
//             //                         "tx_accumulator_num_nodes",
//             //                     ));
//             //                 }
//             //                 // tx_accumulator_num_nodes = map.next_entry()?.map(|(_k, v)| v);
//             //                 if map.size_hint().unwrap_or(0) > 0 {
//             //                     tx_accumulator_num_nodes = Some(map.next_value()?);
//             //                 }
//             //             }
//             //         }
//             //     }
//
//             // let tx_accumulator_root = H256::from_slice(tx_accumulator_root_bytes.as_slice());
//             // let tx_accumulator_frozen_subtree_roots = tx_accumulator_frozen_subtree_roots_bytes
//             //     .into_iter()
//             //     .map(|v| H256::from_slice(v.as_slice()))
//             //     .collect();
//
//             //     let tx_order =
//             //         tx_order.ok_or_else(|| serde::de::Error::missing_field("tx_order"))?;
//             //     let tx_order_signature = tx_order_signature
//             //         .ok_or_else(|| serde::de::Error::missing_field("tx_order_signature"))?;
//             //     let tx_accumulator_root = tx_accumulator_root
//             //         .ok_or_else(|| serde::de::Error::missing_field("tx_accumulator_root"))?;
//             //     let tx_timestamp =
//             //         tx_timestamp.ok_or_else(|| serde::de::Error::missing_field("tx_timestamp"))?;
//             //
//             //     let tx_accumulator_frozen_subtree_roots =
//             //         tx_accumulator_frozen_subtree_roots.unwrap_or(vec![]);
//             //     let tx_accumulator_num_leaves = tx_accumulator_num_leaves.unwrap_or(0u64);
//             //     let tx_accumulator_num_nodes = tx_accumulator_num_nodes.unwrap_or(0u64);
//             //     Ok(TransactionSequenceInfo {
//             //         tx_order,
//             //         tx_order_signature,
//             //         tx_accumulator_root,
//             //         tx_timestamp,
//             //         tx_accumulator_frozen_subtree_roots,
//             //         tx_accumulator_num_leaves,
//             //         tx_accumulator_num_nodes,
//             //     })
//             // }
//         }
//
//         // const FIELDS: &[&str] = &[
//         //     TX_ORDER_STR,
//         //     TX_ORDER_SIGNATURE_STR,
//         //     TX_ACCUMULATOR_ROOT_STR,
//         //     TX_TIMESTAMP_STR,
//         //     TX_ACCUMULATOR_FROZEN_SUBTREE_ROOTS_STR,
//         //     TX_ACCUMULATOR_NUM_LEAVES_STR,
//         //     TX_ACCUMULATOR_NUM_NODES_STR,
//         // ];
//         deserializer.deserialize_struct(
//             TRANSACTION_SEQUENCE_INFO_STR,
//             TRANSACTION_SEQUENCE_INFO_FIELDS,
//             TransactionSequenceInfoVisitor,
//         )
//     }
// }

/// Transaction with sequence info and execution info.
#[derive(Debug, Clone)]
pub struct TransactionWithInfo {
    pub transaction: LedgerTransaction,
    pub execution_info: Option<TransactionExecutionInfo>,
}

impl TransactionWithInfo {
    pub fn new(ledger_tx: LedgerTransaction, indexer_tx: IndexerTransaction) -> Result<Self> {
        let status: KeptVMStatus = serde_json::from_str(indexer_tx.status.as_str())?;
        let execution_info = TransactionExecutionInfo {
            tx_hash: indexer_tx.tx_hash,
            state_root: indexer_tx.state_root,
            size: indexer_tx.size,
            event_root: indexer_tx.event_root,
            gas_used: indexer_tx.gas_used,
            status,
        };
        Ok(TransactionWithInfo {
            transaction: ledger_tx,
            execution_info: Some(execution_info),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ExecuteTransactionResponse {
    pub sequence_info: TransactionSequenceInfo,
    pub execution_info: TransactionExecutionInfo,
    pub output: TransactionOutput,
}

#[cfg(test)]
mod tests {
    use super::rooch::RoochTransaction;
    use crate::test_utils::random_accumulator_info;
    use crate::transaction::TransactionSequenceInfo;
    use ethers::types::H256;
    use moveos_types::state::MoveState;
    use moveos_types::test_utils::random_bytes;

    fn test_serialize_deserialize_roundtrip(tx: RoochTransaction) {
        let bytes = tx.encode();
        let tx2 = RoochTransaction::decode(&bytes).unwrap();
        assert_eq!(tx, tx2);
    }

    #[test]
    fn test_serialize_deserialize() {
        let tx = RoochTransaction::mock();
        test_serialize_deserialize_roundtrip(tx)
    }

    #[test]
    fn test_serialize_deserialize_transaction_sequence_info() {
        let tx_order_signature = random_bytes();
        let accumulator_info = random_accumulator_info();
        let tx_sequence_info =
            TransactionSequenceInfo::new(rand::random(), tx_order_signature, accumulator_info, 0);
        let bcs_bytes = tx_sequence_info.to_bytes();
        let h256_bcs_bytes =
            bcs::to_bytes(&H256::random()).expect("Serialize the H256 should success");
        println!("Serialize transaction sequence info: {:?}", bcs_bytes);
        println!(
            "Serialize transaction sequence info H256: {:?}, len: {}",
            h256_bcs_bytes,
            h256_bcs_bytes.len()
        );
    }
}
