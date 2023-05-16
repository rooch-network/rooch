// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    authenticator::{AccountPrivateKey, Authenticator},
    AbstractTransaction, AuthenticatorInfo, TransactionType,
};
use crate::address::RoochAddress;
use crate::H256;
use anyhow::Result;
use moveos_types::transaction::{MoveAction, MoveOSTransaction};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RoochTransactionData {
    /// Sender's address.
    pub sender: RoochAddress,
    // Sequence number of this transaction corresponding to sender's account.
    pub sequence_number: u64,
    // The MoveAction to execute.
    pub action: MoveAction,
    //TODO how to define Gas paramter and AppID(Or ChainID)
}

impl RoochTransactionData {
    pub fn new(sender: RoochAddress, sequence_number: u64, action: MoveAction) -> Self {
        Self {
            sender,
            sequence_number,
            action,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("encode transaction should success")
    }

    pub fn hash(&self) -> H256 {
        moveos_types::h256::sha3_256_of(self.encode().as_slice())
    }

    /// Signs the given `RoochTransactionData` into RoochTransaction.
    pub fn sign(self, private_key: &AccountPrivateKey) -> Result<RoochTransaction> {
        let msg = self.hash();
        let authenticator = private_key.sign(msg.as_bytes());
        Ok(RoochTransaction {
            data: self,
            authenticator,
        })
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RoochTransaction {
    data: RoochTransactionData,
    authenticator: Authenticator,
}

impl RoochTransaction {
    pub fn new(data: RoochTransactionData, authenticator: Authenticator) -> Self {
        Self {
            data,
            authenticator,
        }
    }

    pub fn sender(&self) -> RoochAddress {
        self.data.sender
    }

    pub fn sequence_number(&self) -> u64 {
        self.data.sequence_number
    }

    pub fn action(&self) -> &MoveAction {
        &self.data.action
    }

    //TODO use protest Arbitrary to generate mock data
    #[cfg(test)]
    pub fn mock() -> RoochTransaction {
        use crate::address::RoochSupportedAddress;
        use move_core_types::{
            account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
        };

        let sender = RoochAddress::random();
        let sequence_number = 0;
        let payload = MoveAction::new_function(
            ModuleId::new(AccountAddress::random(), Identifier::new("test").unwrap()),
            Identifier::new("test").unwrap(),
            vec![],
            vec![],
        );

        let transaction_data = RoochTransactionData::new(sender, sequence_number, payload);
        let private_key = AccountPrivateKey::generate_for_testing();
        transaction_data.sign(&private_key).unwrap()
    }
}

impl AbstractTransaction for RoochTransaction {
    type Hash = H256;

    fn transaction_type(&self) -> super::TransactionType {
        TransactionType::Rooch
    }

    fn decode(bytes: &[u8]) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        bcs::from_bytes::<Self>(bytes).map_err(Into::into)
    }

    fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("encode transaction should success")
    }

    fn tx_hash(&self) -> Self::Hash {
        //TODO cache the hash
        moveos_types::h256::sha3_256_of(self.encode().as_slice())
    }

    fn authenticator(&self) -> AuthenticatorInfo {
        AuthenticatorInfo {
            sender: self.sender().into(),
            authenticator: self.authenticator.clone(),
        }
    }
}

impl From<RoochTransaction> for MoveOSTransaction {
    fn from(tx: RoochTransaction) -> Self {
        let tx_hash = tx.tx_hash();
        MoveOSTransaction::new(tx.data.sender.into(), tx.data.action, tx_hash)
    }
}
