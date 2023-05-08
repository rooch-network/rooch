// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ethereum::{EthereumSignature, EthereumTransactionData};

pub mod ethereum;

pub enum TransactionType {
    Ethereum,
    Move,
}

pub type RawTransaction = TransactionEnvelope<Vec<u8>, Vec<u8>>;
pub type EthereumTransaction = TransactionEnvelope<EthereumTransactionData, EthereumSignature>;

//TODO think about how to define signature.
pub struct TransactionEnvelope<T, S> {
    pub transaction_type: TransactionType,
    pub data: T,
    pub auth_signature: S,
}
