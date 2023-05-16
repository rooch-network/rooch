// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::AbstractTransaction;
use anyhow::Result;
use ethers::utils::rlp::{Decodable, Rlp};
use moveos_types::{h256::H256, transaction::MoveOSTransaction};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EthereumTransaction(pub ethers::core::types::Transaction);

impl AbstractTransaction for EthereumTransaction {
    type Authenticator = ethers::core::types::Signature;
    type Hash = H256;

    fn transaction_type(&self) -> super::TransactionType {
        super::TransactionType::Ethereum
    }

    fn authenticator(&self) -> Self::Authenticator {
        ethers::core::types::Signature {
            r: self.0.r,
            s: self.0.s,
            v: self.0.v.as_u64(),
        }
    }

    fn decode(bytes: &[u8]) -> Result<Self> {
        let rlp = Rlp::new(bytes);
        let mut txn = ethers::core::types::Transaction::decode(&rlp)?;
        txn.recover_from_mut()?;
        Ok(Self(txn))
    }

    fn encode(&self) -> Vec<u8> {
        self.0.rlp().to_vec()
    }

    fn tx_hash(&self) -> Self::Hash {
        self.0.hash()
    }

    fn verify(&self) -> bool {
        todo!("verify ethereum transaction")
    }
}

impl From<EthereumTransaction> for MoveOSTransaction {
    fn from(_tx: EthereumTransaction) -> Self {
        todo!("convert ethereum transaction to moveos transaction")
    }
}
