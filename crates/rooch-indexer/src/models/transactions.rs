// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::schema::transactions;
use diesel::prelude::*;
use move_core_types::account_address::AccountAddress;
use moveos_types::h256::H256;
use rooch_types::indexer::transaction::IndexerTransaction;
use std::str::FromStr;

#[derive(Clone, Debug, Queryable, Insertable, QueryableByName)]
#[diesel(table_name = transactions)]
pub struct StoredTransaction {
    /// The hash of this transaction.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub tx_hash: String,
    /// The tx order of this transaction.
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub tx_order: i64,

    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub sequence_number: i64,
    /// the rooch address of sender who send the transaction
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub sender: String,
    #[diesel(sql_type = diesel::sql_types::SmallInt)]
    pub action_type: i16,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub auth_validator_id: i64,
    #[diesel(sql_type = diesel::sql_types::Blob)]
    pub authenticator_payload: Vec<u8>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub tx_accumulator_root: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub state_root: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub size: i64,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub event_root: String,
    /// The amount of gas used.
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub gas_used: i64,
    /// The vm status.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub status: String,

    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub created_at: i64,
}

impl From<IndexerTransaction> for StoredTransaction {
    fn from(transaction: IndexerTransaction) -> Self {
        StoredTransaction {
            tx_hash: format!("{:?}", transaction.tx_hash),
            tx_order: transaction.tx_order as i64,
            sequence_number: transaction.sequence_number as i64,
            sender: transaction.sender.to_hex_literal(),
            action_type: transaction.action_type as i16,
            auth_validator_id: transaction.auth_validator_id as i64,
            authenticator_payload: transaction.authenticator_payload,
            tx_accumulator_root: format!("{:?}", transaction.tx_accumulator_root),
            state_root: format!("{:?}", transaction.state_root),
            size: transaction.size as i64,
            event_root: format!("{:?}", transaction.event_root),
            gas_used: transaction.gas_used as i64,
            status: transaction.status,
            created_at: transaction.created_at as i64,
        }
    }
}

impl TryFrom<StoredTransaction> for IndexerTransaction {
    type Error = anyhow::Error;

    fn try_from(transaction: StoredTransaction) -> Result<Self, Self::Error> {
        let sender = AccountAddress::from_hex_literal(transaction.sender.as_str())?;
        let tx_hash = H256::from_str(transaction.tx_hash.as_str())?;
        let tx_accumulator_root = H256::from_str(transaction.tx_accumulator_root.as_str())?;
        let state_root = H256::from_str(transaction.state_root.as_str())?;
        let event_root = H256::from_str(transaction.event_root.as_str())?;

        let indexer_transaction = IndexerTransaction {
            tx_hash,
            tx_order: transaction.tx_order as u64,
            sequence_number: transaction.sequence_number as u64,
            sender,
            action: None,
            action_type: transaction.action_type as u8,
            auth_validator_id: transaction.auth_validator_id as u64,
            authenticator_payload: transaction.authenticator_payload,
            tx_accumulator_root,
            state_root,
            size: transaction.size as u64,
            event_root,
            gas_used: transaction.gas_used as u64,
            status: transaction.status,
            created_at: transaction.created_at as u64,
        };
        Ok(indexer_transaction)
    }
}
