// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_types::transaction::{TransactionType, TypedTransaction};

use moveos_types::transaction::MoveAction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionView {
    pub transaction_type: TransactionType,
    // Sequence number of this transaction corresponding to sender's account.
    pub sequence_number: u64,
    pub sender: String,
    // The MoveAction to execute.
    pub action: MoveAction,
}

impl From<TypedTransaction> for TransactionView {
    fn from(transaction: TypedTransaction) -> Self {
        let transaction_type = transaction.transaction_type();
        match transaction {
            TypedTransaction::Rooch(rooch) => Self {
                transaction_type,
                sequence_number: rooch.sequence_number(),
                sender: rooch.sender().to_string(),
                action: rooch.action().clone(),
            },
            TypedTransaction::Ethereum(eth) => Self {
                transaction_type,
                sequence_number: eth.0.nonce.as_u64(),
                sender: eth.0.from.to_string(),
                action: eth.decode_calldata_to_action().unwrap(),
            },
        }
    }
}
