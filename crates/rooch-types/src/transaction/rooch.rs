// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    authenticator::{AccountPrivateKey, Authenticator},
    AbstractTransaction, TransactionType,
};
use crate::address::RoochAddress;
use crate::H256;
use anyhow::Result;
use moveos_types::transaction::MoveTransaction;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RoochTransactionData {
    /// Sender's address.
    pub sender: RoochAddress,
    // Sequence number of this transaction corresponding to sender's account.
    pub sequence_number: u64,
    // The transaction script to execute.
    pub payload: MoveTransaction,
    //TODO how to define Gas paramter and AppID(Or ChainID)
}

impl RoochTransactionData {
    pub fn new(sender: RoochAddress, sequence_number: u64, payload: MoveTransaction) -> Self {
        Self {
            sender,
            sequence_number,
            payload,
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
    #[serde(flatten)]
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
}

impl AbstractTransaction for RoochTransaction {
    type Authenticator = Authenticator;
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

    fn hash(&self) -> Self::Hash {
        //TODO cache the hash
        moveos_types::h256::sha3_256_of(self.encode().as_slice())
    }

    fn authenticator(&self) -> Self::Authenticator {
        self.authenticator.clone()
    }

    fn verify(&self) -> bool {
        self.authenticator
            .verify(self.data.hash().as_bytes())
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use move_core_types::{
        account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
    };

    use crate::address::RoochSupportedAddress;

    use super::*;

    #[test]
    fn test_rooch_transaction() {
        let sender = RoochAddress::random();
        let sequence_number = 0;
        let payload = MoveTransaction::new_function(
            ModuleId::new(AccountAddress::random(), Identifier::new("test").unwrap()),
            Identifier::new("test").unwrap(),
            vec![],
            vec![],
        );

        let transaction_data = RoochTransactionData::new(sender, sequence_number, payload);
        let private_key = AccountPrivateKey::generate_for_testing();
        let transaction = transaction_data.sign(&private_key).unwrap();
        assert!(transaction.verify());
    }
}
