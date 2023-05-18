// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    authenticator::Authenticator, AbstractTransaction, AuthenticatorInfo, AuthenticatorResult,
};
use crate::address::EthereumAddress;
use anyhow::Result;
use ethers::utils::rlp::{Decodable, Rlp};
use moveos_types::{
    h256::H256,
    transaction::{AuthenticatableTransaction, MoveAction, MoveOSTransaction},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EthereumTransaction(pub ethers::core::types::Transaction);

impl EthereumTransaction {
    //This function is just a demo, we should define the Ethereum calldata's MoveAction standard
    pub fn decode_calldata_to_action(&self) -> Result<MoveAction> {
        //Maybe we should use RLP to encode the MoveAction
        bcs::from_bytes(&self.0.input)
            .map_err(|e| anyhow::anyhow!("decode calldata to action failed: {}", e))
    }
}

impl AbstractTransaction for EthereumTransaction {
    type Hash = H256;

    fn transaction_type(&self) -> super::TransactionType {
        super::TransactionType::Ethereum
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
}

impl AuthenticatableTransaction for EthereumTransaction {
    type AuthenticatorInfo = AuthenticatorInfo;
    type AuthenticatorResult = AuthenticatorResult;

    fn tx_hash(&self) -> H256 {
        self.0.hash()
    }

    fn authenticator_info(&self) -> AuthenticatorInfo {
        AuthenticatorInfo {
            sender: EthereumAddress(self.0.from).into(),
            //TODO should change the seqence_number to u256?
            seqence_number: self.0.nonce.as_u64(),
            authenticator: Authenticator::secp256k1(ethers::core::types::Signature {
                r: self.0.r,
                s: self.0.s,
                v: self.0.v.as_u64(),
            }),
        }
    }

    fn construct_moveos_transaction(
        &self,
        result: super::AuthenticatorResult,
    ) -> Result<MoveOSTransaction> {
        let action = self.decode_calldata_to_action()?;
        Ok(MoveOSTransaction::new(
            result.resolved_address,
            action,
            self.tx_hash(),
        ))
    }
}
