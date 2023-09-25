// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{authenticator::Authenticator, AbstractTransaction, AuthenticatorInfo};
use crate::{
    address::EthereumAddress, chain_id::RoochChainID, error::RoochError,
    framework::auth_validator::BuiltinAuthValidator,
};
use anyhow::Result;
use ethers::{
    types::{Bytes, OtherFields, Signature, Transaction, U256, U64},
    utils::rlp::{Decodable, Rlp},
};
use move_core_types::account_address::AccountAddress;
use moveos_types::{
    gas_config::GasConfig,
    h256::{self, H256},
    transaction::{MoveAction, MoveOSTransaction},
    tx_context::TxContext,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EthereumTransactionData(pub Transaction);

impl EthereumTransactionData {
    pub fn new_for_test(sender: EthereumAddress, nonce: U256, action: Bytes) -> Self {
        let sender_and_action = (sender, action.clone());
        let tx_hash = h256::sha3_256_of(bcs::to_bytes(&sender_and_action).unwrap().as_slice());
        let transaction = Transaction {
            hash: tx_hash,
            nonce,
            block_hash: None,
            block_number: None,
            transaction_index: None,
            from: sender.0,
            to: None,
            value: U256::one(),
            gas_price: None,
            gas: GasConfig::DEFAULT_MAX_GAS_AMOUNT.into(),
            input: action,
            v: U64::one(),
            r: U256::one(),
            s: U256::one(),
            transaction_type: None,
            access_list: None,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            chain_id: Some(U256::from(RoochChainID::LOCAL.chain_id().id())), // build ethereum chain id from rooch chain id as parsed from MetaMask rooch chain id
            other: OtherFields::default(),
        };

        Self(transaction)
    }

    //This function is just a demo, we should define the Ethereum calldata's MoveAction standard
    pub fn decode_calldata_to_action(&self) -> Result<MoveAction> {
        //Maybe we should use RLP to encode the MoveAction
        bcs::from_bytes(&self.0.input)
            .map_err(|e| anyhow::anyhow!("decode calldata to action failed: {}", e))
    }

    pub fn into_signature(&self) -> Result<Signature, RoochError> {
        // Extract signature from original transaction
        let r = self.0.r;
        let s = self.0.s;
        let v = self.0.v.as_u64();

        // Keep original Ethereum signature
        Ok(Signature { r, s, v })
    }

    pub fn into_address(&self) -> Result<EthereumAddress, RoochError> {
        // Prepare the signed message (RLP encoding of the transaction)
        let message = self.tx_hash().to_fixed_bytes();
        // Get the signature
        let ethereum_signature = self.into_signature()?;
        // Recover the h160 address using default recover method
        let h160_address = ethereum_signature
            .recover(message)
            .expect("Recover to an Ethereum address should succeed");
        // Get the address
        let ethereum_address = EthereumAddress(h160_address);

        Ok(ethereum_address)
    }
}

impl AbstractTransaction for EthereumTransactionData {
    fn transaction_type(&self) -> super::TransactionType {
        super::TransactionType::Ethereum
    }

    fn decode(bytes: &[u8]) -> Result<Self> {
        let rlp = Rlp::new(bytes);
        let mut tx = ethers::core::types::Transaction::decode(&rlp)?;
        tx.recover_from_mut()?;
        Ok(Self(tx))
    }

    fn encode(&self) -> Vec<u8> {
        self.0.rlp().to_vec()
    }

    fn tx_hash(&self) -> H256 {
        self.0.hash()
    }

    fn construct_moveos_transaction(
        self,
        resolved_sender: AccountAddress,
    ) -> Result<MoveOSTransaction> {
        let action = self.decode_calldata_to_action()?;
        let sequence_number = self.0.nonce.as_u64();
        let gas = self.0.gas.as_u64();
        let tx_ctx = TxContext::new(resolved_sender, sequence_number, gas, self.tx_hash());
        Ok(MoveOSTransaction::new(tx_ctx, action))
    }

    fn sender(&self) -> crate::address::MultiChainAddress {
        EthereumAddress(self.0.from).into()
    }

    fn authenticator_info(&self) -> Result<AuthenticatorInfo> {
        let chain_id = self.0.chain_id.ok_or(RoochError::InvalidChainID)?.as_u64();
        let authenticator = Authenticator::new(
            BuiltinAuthValidator::Ethereum.flag().into(),
            self.into_signature()?.to_vec(),
        );
        Ok(AuthenticatorInfo::new(chain_id, authenticator))
    }
}
