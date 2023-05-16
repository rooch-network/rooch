// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use self::authenticator::Authenticator;
use crate::{address::MultiChainAddress, H256};
use anyhow::Result;
use moveos_types::transaction::MoveOSTransaction;
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
    pub sender: MultiChainAddress,
    pub authenticator: Authenticator,
}

impl AuthenticatorInfo {
    pub fn new(sender: MultiChainAddress, authenticator: Authenticator) -> Self {
        Self {
            sender,
            authenticator,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("encode authenticator info should success")
    }
}

impl From<AuthenticatorInfo> for Vec<u8> {
    fn from(info: AuthenticatorInfo) -> Self {
        info.encode()
    }
}

pub trait AbstractTransaction {
    /// The transaction sender authenticator type.
    /// Usually it is a signature.
    type Hash;

    fn transaction_type(&self) -> TransactionType;

    fn decode(bytes: &[u8]) -> Result<Self>
    where
        Self: std::marker::Sized;
    fn encode(&self) -> Vec<u8>;

    fn tx_hash(&self) -> Self::Hash;
    fn authenticator(&self) -> AuthenticatorInfo;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TypedTransaction {
    Rooch(rooch::RoochTransaction),
    Ethereum(ethereum::EthereumTransaction),
}

impl TypedTransaction {
    pub fn transaction_type(&self) -> TransactionType {
        match self {
            TypedTransaction::Rooch(_) => TransactionType::Rooch,
            TypedTransaction::Ethereum(_) => TransactionType::Ethereum,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        match self {
            TypedTransaction::Rooch(tx) => tx.encode(),
            TypedTransaction::Ethereum(tx) => tx.encode(),
        }
    }

    pub fn hash(&self) -> H256 {
        match self {
            TypedTransaction::Rooch(tx) => tx.tx_hash(),
            TypedTransaction::Ethereum(tx) => tx.tx_hash(),
        }
    }
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
                let tx = ethereum::EthereumTransaction::decode(&raw.raw)?;
                Ok(TypedTransaction::Ethereum(tx))
            }
        }
    }
}

impl From<TypedTransaction> for MoveOSTransaction {
    fn from(tx: TypedTransaction) -> Self {
        match tx {
            TypedTransaction::Rooch(tx) => tx.into(),
            TypedTransaction::Ethereum(tx) => tx.into(),
        }
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
