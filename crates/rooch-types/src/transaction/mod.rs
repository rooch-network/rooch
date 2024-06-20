// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use framework_types::addresses::ROOCH_FRAMEWORK_ADDRESS;
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::vm_status::KeptVMStatus;
use moveos_types::state::{MoveStructState, MoveStructType};
use moveos_types::transaction::TransactionExecutionInfo;
use moveos_types::{h256::H256, transaction::TransactionOutput};
use serde::{Deserialize, Serialize};

pub mod authenticator;
mod ledger_transaction;
pub mod rooch;

use crate::indexer::transaction::IndexerTransaction;
pub use authenticator::Authenticator;
pub use ledger_transaction::{
    L1Block, L1BlockWithBody, L1Transaction, LedgerTransaction, LedgerTxData,
};
pub use rooch::{RoochTransaction, RoochTransactionData};

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
    /// The timestamp of the sequencer when the tx is sequenced, in millisecond.
    pub tx_timestamp: u64,
}

impl TransactionSequenceInfo {
    pub fn new(
        tx_order: u64,
        tx_order_signature: Vec<u8>,
        tx_accumulator_root: H256,
        tx_timestamp: u64,
    ) -> TransactionSequenceInfo {
        TransactionSequenceInfo {
            tx_order,
            tx_order_signature,
            tx_accumulator_root,
            tx_timestamp,
        }
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
        ])
    }
}

/// Transaction with sequence info and execution info.
#[derive(Debug, Clone)]
pub struct TransactionWithInfo {
    pub transaction: LedgerTransaction,
    pub execution_info: TransactionExecutionInfo,
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
            execution_info,
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
}
