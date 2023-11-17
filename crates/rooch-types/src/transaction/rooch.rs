// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    authenticator::Authenticator, AbstractTransaction, AuthenticatorInfo, TransactionType,
};
use crate::crypto::{Ed25519RoochSignature, RoochKeyPair, Signature};
use crate::multichain_id::{MultiChainID, ROOCH};
use crate::{address::RoochAddress, chain_id::RoochChainID};
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use moveos_types::gas_config::GasConfig;
use moveos_types::h256::H256;
use moveos_types::{
    moveos_std::tx_context::TxContext,
    transaction::{MoveAction, MoveOSTransaction},
};
use serde::{Deserialize, Serialize};
use std::debug_assert;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RoochTransactionData {
    /// Sender's address.
    pub sender: RoochAddress,
    // Sequence number of this transaction corresponding to sender's account.
    pub sequence_number: u64,
    // The ChainID of the transaction.
    pub chain_id: u64,
    // The max gas to be used.
    pub max_gas_amount: u64,
    // The MoveAction to execute.
    pub action: MoveAction,
}

impl RoochTransactionData {
    pub fn new(
        sender: RoochAddress,
        sequence_number: u64,
        chain_id: u64,
        max_gas_amount: u64,
        action: MoveAction,
    ) -> Self {
        Self {
            sender,
            sequence_number,
            chain_id,
            max_gas_amount,
            action,
        }
    }

    pub fn new_for_test(sender: RoochAddress, sequence_number: u64, action: MoveAction) -> Self {
        Self {
            sender,
            sequence_number,
            chain_id: RoochChainID::LOCAL.chain_id().id(),
            max_gas_amount: GasConfig::DEFAULT_MAX_GAS_AMOUNT,
            action,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("encode transaction should success")
    }

    pub fn hash(&self) -> H256 {
        moveos_types::h256::sha3_256_of(self.encode().as_slice())
    }

    pub fn sign(self, kp: &RoochKeyPair) -> RoochTransaction {
        let signature = Signature::new_hashed(self.hash().as_bytes(), kp);
        //TODO implement Signature into Authenticator
        let authenticator = Authenticator::rooch(signature);
        RoochTransaction::new(self, authenticator)
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

    pub fn new_genesis_tx(
        genesis_address: RoochAddress,
        chain_id: u64,
        action: MoveAction,
    ) -> Self {
        Self {
            data: RoochTransactionData::new(genesis_address, 0, chain_id, u64::max_value(), action),
            authenticator: Authenticator::rooch(Signature::Ed25519RoochSignature(
                Ed25519RoochSignature::default(),
            )),
        }
    }

    pub fn sender(&self) -> RoochAddress {
        self.data.sender
    }

    pub fn sequence_number(&self) -> u64 {
        self.data.sequence_number
    }

    pub fn chain_id(&self) -> u64 {
        self.data.chain_id
    }

    pub fn max_gas_amount(&self) -> u64 {
        self.data.max_gas_amount
    }

    pub fn action(&self) -> &MoveAction {
        &self.data.action
    }

    //TODO use protest Arbitrary to generate mock data
    #[cfg(test)]
    pub fn mock() -> RoochTransaction {
        use crate::address::RoochSupportedAddress;
        use fastcrypto::ed25519::Ed25519KeyPair;
        use fastcrypto::traits::KeyPair;
        use move_core_types::{identifier::Identifier, language_storage::ModuleId};
        use moveos_types::move_types::FunctionId;

        let sender: RoochAddress = RoochAddress::random();
        let sequence_number = 0;
        let payload = MoveAction::new_function_call(
            FunctionId::new(
                ModuleId::new(AccountAddress::random(), Identifier::new("test").unwrap()),
                Identifier::new("test").unwrap(),
            ),
            vec![],
            vec![],
        );

        let transaction_data = RoochTransactionData::new_for_test(sender, sequence_number, payload);
        let mut rng = rand::thread_rng();
        let ed25519_keypair: Ed25519KeyPair = Ed25519KeyPair::generate(&mut rng);
        let auth =
            Signature::new_hashed(transaction_data.hash().as_bytes(), &ed25519_keypair).into();
        RoochTransaction::new(transaction_data, auth)
    }
}

impl From<RoochTransaction> for MoveOSTransaction {
    fn from(tx: RoochTransaction) -> Self {
        let tx_hash = tx.tx_hash();
        let tx_ctx = TxContext::new(
            tx.data.sender.into(),
            tx.data.sequence_number,
            tx.data.max_gas_amount,
            tx_hash,
        );
        MoveOSTransaction::new(tx_ctx, tx.data.action)
    }
}

impl AbstractTransaction for RoochTransaction {
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

    //TODO unify the hash function
    fn tx_hash(&self) -> H256 {
        //TODO cache the hash
        self.data.hash()
    }

    fn authenticator_info(&self) -> Result<AuthenticatorInfo> {
        Ok(AuthenticatorInfo::new(
            self.chain_id(),
            self.authenticator.clone(),
        ))
    }

    fn construct_moveos_transaction(
        self,
        resolved_sender: AccountAddress,
    ) -> Result<MoveOSTransaction> {
        debug_assert!(self.sender() == resolved_sender.into());
        Ok(self.into())
    }

    fn sender(&self) -> crate::address::MultiChainAddress {
        self.sender().into()
    }

    fn original_address_str(&self) -> String {
        self.data.sender.to_string()
    }

    fn multi_chain_id(&self) -> MultiChainID {
        MultiChainID::from(ROOCH)
    }
}
