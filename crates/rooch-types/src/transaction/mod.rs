// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::transaction::TransactionExecutionInfo;
use moveos_types::{h256::H256, transaction::TransactionOutput};
use serde::{Deserialize, Serialize};

pub mod authenticator;
mod ledger_transaction;
pub mod rooch;

pub use authenticator::Authenticator;
pub use ledger_transaction::{L1Block, L1BlockWithBody, LedgerTransaction, LedgerTxData};
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
    pub tx_order_signature: Authenticator,
    /// The tx accumulator root after the tx is append to the accumulator.
    pub tx_accumulator_root: H256,
}

impl TransactionSequenceInfo {
    pub fn new(
        tx_order: u64,
        tx_order_signature: Authenticator,
        tx_accumulator_root: H256,
    ) -> TransactionSequenceInfo {
        TransactionSequenceInfo {
            tx_order,
            tx_order_signature,
            tx_accumulator_root,
        }
    }
}

/// Transaction with sequence info and execution info.
#[derive(Debug, Clone)]
pub struct TransactionWithInfo {
    pub transaction: RoochTransaction,
    pub sequence_info: TransactionSequenceInfo,
    pub execution_info: TransactionExecutionInfo,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionSequenceInfoMapping {
    /// The tx order
    pub tx_order: u64,
    /// The tx hash.
    pub tx_hash: H256,
}

impl TransactionSequenceInfoMapping {
    pub fn new(tx_order: u64, tx_hash: H256) -> TransactionSequenceInfoMapping {
        TransactionSequenceInfoMapping { tx_order, tx_hash }
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
