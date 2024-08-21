// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::schema::transactions;
use crate::utils::escape_sql_string;
use diesel::prelude::*;
use moveos_types::h256::H256;
use rooch_types::address::RoochAddress;
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
            created_at: transaction.created_at as i64,
        }
    }
}

impl TryFrom<StoredTransaction> for IndexerTransaction {
    type Error = anyhow::Error;

    fn try_from(transaction: StoredTransaction) -> Result<Self, Self::Error> {
        let sender = RoochAddress::from_hex_literal(transaction.sender.as_str())?;
        let tx_hash = H256::from_str(transaction.tx_hash.as_str())?;

        let indexer_transaction = IndexerTransaction {
            tx_hash,
            tx_order: transaction.tx_order as u64,
            sequence_number: transaction.sequence_number as u64,
            sender,
            action_type: transaction.action_type as u8,
            auth_validator_id: transaction.auth_validator_id as u64,
            created_at: transaction.created_at as u64,
        };
        Ok(indexer_transaction)
    }
}

pub fn escape_transaction(mut transaction: StoredTransaction) -> StoredTransaction {
    transaction.sender = escape_sql_string(transaction.sender.clone());
    transaction
}
