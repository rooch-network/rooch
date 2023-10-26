// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{authenticator::Authenticator, AbstractTransaction, AuthenticatorInfo};
use crate::{
    address::{EthereumAddress, RoochAddress},
    chain_id::RoochChainID,
    error::RoochError,
    framework::{
        auth_validator::BuiltinAuthValidator, gas_coin::GasCoin, transfer::TransferModule,
    },
};
use anyhow::{bail, Result};
use ethers::{
    types::{
        transaction::eip2930::AccessList, Bytes, OtherFields, Signature, Transaction, H160, U256,
        U64,
    },
    utils::rlp::{Decodable, Rlp},
};
use move_core_types::account_address::AccountAddress;
use moveos_types::{
    gas_config::GasConfig,
    h256::{self, H256},
    moveos_std::tx_context::TxContext,
    state::MoveStructType,
    transaction::{MoveAction, MoveOSTransaction},
};
use serde::{Deserialize, Serialize};

// TODO: Remove EthereumTransaction and only keep Transaction body
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EthereumTransaction(pub Transaction);

pub struct EthereumTransactionBuilder {
    hash: H256,
    nonce: U256,
    block_hash: Option<H256>,
    block_number: Option<U64>,
    transaction_index: Option<U64>,
    from: H160,
    to: Option<H160>,
    value: U256,
    gas_price: Option<U256>,
    gas: U256,
    input: Bytes,
    v: U64,
    r: U256,
    s: U256,
    transaction_type: Option<U64>,
    access_list: Option<AccessList>,
    max_priority_fee_per_gas: Option<U256>,
    max_fee_per_gas: Option<U256>,
    chain_id: Option<U256>,
    other: OtherFields,
}

impl EthereumTransactionBuilder {
    pub fn new() -> Self {
        EthereumTransactionBuilder {
            hash: Default::default(),
            nonce: Default::default(),
            block_hash: None,
            block_number: None,
            transaction_index: None,
            from: Default::default(),
            to: None,
            value: Default::default(),
            gas_price: None,
            gas: Default::default(),
            input: Default::default(),
            v: Default::default(),
            r: Default::default(),
            s: Default::default(),
            transaction_type: None,
            access_list: None,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            chain_id: None,
            other: Default::default(),
        }
    }

    pub fn hash(mut self, hash: H256) -> Self {
        self.hash = hash;
        self
    }

    pub fn nonce(mut self, nonce: U256) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn block_hash(mut self, block_hash: Option<H256>) -> Self {
        self.block_hash = block_hash;
        self
    }

    pub fn block_number(mut self, block_number: Option<U64>) -> Self {
        self.block_number = block_number;
        self
    }

    pub fn transaction_index(mut self, transaction_index: Option<U64>) -> Self {
        self.transaction_index = transaction_index;
        self
    }

    pub fn from(mut self, from: H160) -> Self {
        self.from = from;
        self
    }

    pub fn to(mut self, to: Option<H160>) -> Self {
        self.to = to;
        self
    }

    pub fn value(mut self, value: U256) -> Self {
        self.value = value;
        self
    }

    pub fn gas_price(mut self, gas_price: Option<U256>) -> Self {
        self.gas_price = gas_price;
        self
    }

    pub fn gas(mut self, gas: U256) -> Self {
        self.gas = gas;
        self
    }

    pub fn input(mut self, input: Bytes) -> Self {
        self.input = input;
        self
    }

    pub fn v(mut self, v: U64) -> Self {
        self.v = v;
        self
    }

    pub fn r(mut self, r: U256) -> Self {
        self.r = r;
        self
    }

    pub fn s(mut self, s: U256) -> Self {
        self.s = s;
        self
    }

    pub fn transaction_type(mut self, transaction_type: Option<U64>) -> Self {
        self.transaction_type = transaction_type;
        self
    }

    pub fn access_list(mut self, access_list: Option<AccessList>) -> Self {
        self.access_list = access_list;
        self
    }

    pub fn max_priority_fee_per_gas(mut self, max_priority_fee_per_gas: Option<U256>) -> Self {
        self.max_priority_fee_per_gas = max_priority_fee_per_gas;
        self
    }

    pub fn max_fee_per_gas(mut self, max_fee_per_gas: Option<U256>) -> Self {
        self.max_fee_per_gas = max_fee_per_gas;
        self
    }

    pub fn chain_id(mut self, chain_id: Option<U256>) -> Self {
        self.chain_id = chain_id;
        self
    }

    pub fn other(mut self, other: OtherFields) -> Self {
        self.other = other;
        self
    }

    pub fn build(self) -> EthereumTransaction {
        let transaction = Transaction {
            hash: self.hash,
            nonce: self.nonce,
            block_hash: self.block_hash,
            block_number: self.block_number,
            transaction_index: self.transaction_index,
            from: self.from,
            to: self.to,
            value: self.value,
            gas_price: self.gas_price,
            gas: self.gas,
            input: self.input,
            v: self.v,
            r: self.r,
            s: self.s,
            transaction_type: self.transaction_type,
            access_list: self.access_list,
            max_priority_fee_per_gas: self.max_priority_fee_per_gas,
            max_fee_per_gas: self.max_fee_per_gas,
            chain_id: self.chain_id,
            other: self.other,
        };

        EthereumTransaction(transaction)
    }
}

impl Default for EthereumTransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EthereumTransaction {
    pub fn new(&self) -> Self {
        let builder = EthereumTransactionBuilder::new();
        builder
            .nonce(self.0.nonce)
            .hash(self.0.hash)
            .block_hash(self.0.block_hash)
            .block_number(self.0.block_number)
            .transaction_index(self.0.transaction_index)
            .from(self.0.from)
            .to(self.0.to)
            .value(self.0.value)
            .gas_price(self.0.gas_price)
            .gas(self.0.gas)
            .input(self.0.input.clone())
            .v(self.0.v)
            .r(self.0.r)
            .s(self.0.s)
            .transaction_type(self.0.transaction_type)
            .access_list(self.0.access_list.clone())
            .max_priority_fee_per_gas(self.0.max_priority_fee_per_gas)
            .max_fee_per_gas(self.0.max_fee_per_gas)
            .chain_id(self.0.chain_id)
            .other(self.0.other.clone())
            .build()
    }

    pub fn new_for_test(sender: RoochAddress, nonce: U256, action: Bytes) -> Self {
        let sender_and_action = (sender, action.clone());
        let tx_hash = h256::sha3_256_of(bcs::to_bytes(&sender_and_action).unwrap().as_slice());
        let transaction = Transaction {
            hash: tx_hash,
            nonce,
            block_hash: None,
            block_number: None,
            transaction_index: None,
            from: H160::from_slice(&sender.0 .0[..20]), // scrape first 20 bytes as ethereum address
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
        if self.0.input.is_empty() {
            match &self.0.to {
                Some(to) => {
                    let to = EthereumAddress(*to);
                    Ok(
                        TransferModule::create_transfer_coin_to_multichain_address_action(
                            GasCoin::struct_tag(),
                            to.into(),
                            crate::framework::ethereum_light_client::eth_u256_to_move_u256(
                                &self.0.value,
                            ),
                        ),
                    )
                }
                None => {
                    bail!("to address is empty, invalid transaction");
                }
            }
        } else {
            //Maybe we should use RLP to encode the MoveAction
            bcs::from_bytes(&self.0.input)
                .map_err(|e| anyhow::anyhow!("decode calldata to action failed: {}", e))
        }
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

impl AbstractTransaction for EthereumTransaction {
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
