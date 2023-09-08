// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use self::authenticator::Authenticator;
use crate::address::MultiChainAddress;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use moveos_types::{h256::H256, transaction::MoveOSTransaction};
use serde::{Deserialize, Serialize};

pub mod authenticator;
pub mod ethereum;
pub mod rooch;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TransactionType {
    Rooch,
    Ethereum,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RawTransaction {
    pub transaction_type: TransactionType,
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

pub trait AbstractTransaction {
    fn transaction_type(&self) -> TransactionType;

    fn decode(bytes: &[u8]) -> Result<Self>
    where
        Self: std::marker::Sized;
    fn encode(&self) -> Vec<u8>;

    fn sender(&self) -> MultiChainAddress;

    fn tx_hash(&self) -> H256;

    fn authenticator_info(&self) -> Result<AuthenticatorInfo>;

    fn construct_moveos_transaction(
        self,
        resolved_sender: AccountAddress,
    ) -> Result<MoveOSTransaction>;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TypedTransaction {
    Rooch(rooch::RoochTransaction),
    Ethereum(ethereum::EthereumTransactionData),
}

impl TryFrom<RawTransaction> for TypedTransaction {
    type Error = anyhow::Error;

    fn try_from(raw: RawTransaction) -> Result<Self> {
        match raw.transaction_type {
            TransactionType::Rooch => {
                let tx = rooch::RoochTransaction::decode(&raw.raw)?;
                Ok(TypedTransaction::Rooch(tx))
            }
            TransactionType::Ethereum => {
                let tx = ethereum::EthereumTransactionData::decode(&raw.raw)?;
                Ok(TypedTransaction::Ethereum(tx))
            }
        }
    }
}

impl AbstractTransaction for TypedTransaction {
    fn transaction_type(&self) -> TransactionType {
        match self {
            TypedTransaction::Rooch(_) => TransactionType::Rooch,
            TypedTransaction::Ethereum(_) => TransactionType::Ethereum,
        }
    }

    fn encode(&self) -> Vec<u8> {
        match self {
            TypedTransaction::Rooch(tx) => tx.encode(),
            TypedTransaction::Ethereum(tx) => tx.encode(),
        }
    }

    fn decode(bytes: &[u8]) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        let raw = bcs::from_bytes::<RawTransaction>(bytes)?;
        Self::try_from(raw)
    }

    fn tx_hash(&self) -> H256 {
        match self {
            TypedTransaction::Rooch(tx) => tx.tx_hash(),
            TypedTransaction::Ethereum(tx) => tx.tx_hash(),
        }
    }

    fn authenticator_info(&self) -> Result<AuthenticatorInfo> {
        match self {
            TypedTransaction::Rooch(tx) => tx.authenticator_info(),
            TypedTransaction::Ethereum(tx) => tx.authenticator_info(),
        }
    }

    fn construct_moveos_transaction(
        self,
        resolved_sender: AccountAddress,
    ) -> Result<moveos_types::transaction::MoveOSTransaction> {
        match self {
            TypedTransaction::Rooch(tx) => tx.construct_moveos_transaction(resolved_sender),
            TypedTransaction::Ethereum(tx) => tx.construct_moveos_transaction(resolved_sender),
        }
    }

    fn sender(&self) -> MultiChainAddress {
        match self {
            TypedTransaction::Rooch(tx) => AbstractTransaction::sender(tx),
            TypedTransaction::Ethereum(tx) => tx.sender(),
        }
    }
}

///`TransactionSequenceInfo` represents the result of sequence a transaction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionSequenceInfo {
    /// The tx order
    pub tx_order: u128,
    /// The tx order signature, it is the signature of the sequencer to commit the tx order.
    pub tx_order_signature: Authenticator,
    /// The tx accumulator root after the tx is append to the accumulator.
    pub tx_accumulator_root: H256,
}

impl TransactionSequenceInfo {
    pub fn new(
        tx_order: u128,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionSequenceMapping {
    /// The tx order
    pub tx_order: u128,
    /// The tx hash.
    pub tx_hash: H256,
}

impl TransactionSequenceMapping {
    pub fn new(tx_order: u128, tx_hash: H256) -> TransactionSequenceMapping {
        TransactionSequenceMapping { tx_order, tx_hash }
    }
}

#[cfg(test)]
mod tests {
    use super::rooch::RoochTransaction;

    fn test_serialize_deserialize_roundtrip<T>(tx: T)
    where
        T: super::AbstractTransaction + std::fmt::Debug + PartialEq,
    {
        let bytes = tx.encode();
        let tx2 = T::decode(&bytes).unwrap();
        assert_eq!(tx, tx2);
    }

    #[test]
    fn test_serialize_deserialize() {
        let tx = RoochTransaction::mock();
        test_serialize_deserialize_roundtrip(tx)
    }
}
