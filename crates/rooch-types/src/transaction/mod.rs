// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use accumulator::accumulator_info::AccumulatorInfo;
use anyhow::Result;
use framework_types::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use moveos_types::state::{MoveState, MoveStructState, MoveStructType};
use moveos_types::transaction::TransactionExecutionInfo;
use moveos_types::{h256::H256, transaction::TransactionOutput};
use serde::{Deserialize, Serialize};

pub mod authenticator;
mod ledger_transaction;
pub mod rooch;

use crate::test_utils::random_accumulator_info;
pub use authenticator::Authenticator;
pub use ledger_transaction::{
    L1Block, L1BlockWithBody, L1Transaction, LedgerTransaction, LedgerTxData,
};
use moveos_types::test_utils::random_bytes;
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    pub tx_accumulator_frozen_subtree_roots: Vec<H256>,
    /// The total number of leaves in the accumulator.
    pub tx_accumulator_num_leaves: u64,
    /// The total number of nodes in the accumulator.
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

    pub fn random() -> Self {
        TransactionSequenceInfo::new(rand::random(), random_bytes(), random_accumulator_info(), 0)
    }
}

impl MoveStructType for TransactionSequenceInfo {
    const ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;
    const MODULE_NAME: &'static IdentStr = ident_str!("transaction");
    const STRUCT_NAME: &'static IdentStr = ident_str!("TransactionSequenceInfo");
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

/// Transaction with sequence info and execution info.
#[derive(Debug, Clone)]
pub struct TransactionWithInfo {
    pub transaction: LedgerTransaction,
    pub execution_info: Option<TransactionExecutionInfo>,
}

impl TransactionWithInfo {
    pub fn new(
        ledger_tx: LedgerTransaction,
        execution_info: TransactionExecutionInfo,
    ) -> Result<Self> {
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
        let _bcs_bytes = tx_sequence_info.to_bytes();
        let _h256_bcs_bytes =
            bcs::to_bytes(&H256::random()).expect("Serialize the H256 should success");
        // println!("Serialize transaction sequence info: {:?}", _bcs_bytes);
        // println!(
        //     "Serialize transaction sequence info H256: {:?}, len: {}",
        //     h256_bcs_bytes,
        //     _h256_bcs_bytes.len()
        // );
    }
}
