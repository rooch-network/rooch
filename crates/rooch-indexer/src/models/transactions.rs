// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use diesel::prelude::*;
use move_core_types::vm_status::KeptVMStatus;

use crate::schema::transactions;
use crate::types::{IndexedTransaction, IndexerResult};

use moveos_types::transaction::TransactionExecutionInfo;
use rooch_types::transaction::authenticator::Authenticator;
use rooch_types::transaction::TransactionWithInfo;
use rooch_types::transaction::{TransactionSequenceInfo, TypedTransaction};

#[derive(Clone, Debug, Queryable, Insertable, QueryableByName)]
#[diesel(table_name = transactions)]
pub struct StoredTransaction {
    /// The hash of this transaction.
    // pub tx_hash: Varchar(65),
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub tx_hash: String,
    /// The tx order of this transaction.
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub tx_order: i128,

    #[diesel(sql_type = diesel::sql_types::Text)]
    pub transaction_type: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub sequence_number: i64,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub multichain_id: String,
    #[diesel(sql_type = diesel::sql_types::Blob)]
    pub multichain_raw_address: Vec<u8>,
    /// the rooch address of sender who send the transaction
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub sender: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub action: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub action_type: i8,
    #[diesel(sql_type = diesel::sql_types::Blob)]
    pub action_raw: Vec<u8>,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub auth_validator_id: i64,
    #[diesel(sql_type = diesel::sql_types::Blob)]
    pub authenticator_payload: Vec<u8>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub tx_accumulator_root: String,
    #[diesel(sql_type = diesel::sql_types::Blob)]
    pub transaction_raw: Vec<u8>,

    #[diesel(sql_type = diesel::sql_types::Text)]
    pub state_root: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub event_root: String,
    /// The amount of gas used.
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub gas_used: i64,
    /// The vm status.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub status: String,

    /// The tx order signature,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub tx_order_auth_validator_id: i64,
    #[diesel(sql_type = diesel::sql_types::Blob)]
    pub tx_order_authenticator_payload: Vec<u8>,

    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub created_at: i64,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub updated_at: i64,
}

impl From<IndexedTransaction> for StoredTransaction {
    fn from(transaction: IndexedTransaction) -> Self {
        StoredTransaction {
            tx_hash: transaction.tx_hash.to_string(),
            tx_order: transaction.tx_order as i128,
            transaction_type: transaction.transaction_type.transaction_type_name(),
            sequence_number: transaction.sequence_number as i64,
            multichain_id: transaction.multichain_id.multichain_name(),
            multichain_raw_address: transaction.multichain_raw_address,
            sender: transaction.sender.to_string(),
            action: transaction.action.action_name(),
            action_type: transaction.action.action_type() as i8,
            action_raw: transaction.action_raw,
            auth_validator_id: transaction.auth_validator_id as i64,
            authenticator_payload: transaction.authenticator_payload,
            tx_accumulator_root: transaction.tx_accumulator_root.to_string(),
            transaction_raw: transaction.transaction_raw,

            state_root: transaction.state_root.to_string(),
            event_root: transaction.event_root.to_string(),
            gas_used: transaction.gas_used as i64,
            // TODO how to index and display the vm status ?
            status: transaction.status.to_string(),

            tx_order_auth_validator_id: transaction.tx_order_auth_validator_id as i64,
            tx_order_authenticator_payload: transaction.tx_order_authenticator_payload,

            created_at: transaction.created_at as i64,
            updated_at: transaction.updated_at as i64,
        }
    }
}

impl StoredTransaction {
    pub fn try_into_transaction_with_info(self) -> IndexerResult<TransactionWithInfo> {
        //TODO construct TypedTransaction
        let transaction = TypedTransaction {};
        let sequence_info = TransactionSequenceInfo {
            tx_order: self.tx_order as u128,
            tx_order_signature: Authenticator {
                auth_validator_id: self.tx_order_auth_validator_id as u64,
                payload: self.tx_order_authenticator_payload,
            },
            tx_accumulator_root: self.tx_accumulator_root.into(),
        };
        let execution_info = TransactionExecutionInfo {
            tx_hash: self.tx_hash.into(),
            state_root: self.state_root.into(),
            event_root: self.state_root.into(),
            gas_used: self.gas_used as u64,
            //TODO convert KeptVMStatus
            status: KeptVMStatus::Executed,
        };
        Ok(TransactionWithInfo {
            transaction,
            sequence_info,
            execution_info,
        })
    }
}
