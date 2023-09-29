// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::ethereum_address::ETHAddress;
use crate::addresses::ROOCH_FRAMEWORK_ADDRESS;
use anyhow::Result;
use ethers::types::Block;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    u256::{U256, U256_NUM_BYTES},
    value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    transaction::FunctionCall,
    tx_context::TxContext,
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("ethereum_light_client");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Hash of the block
    pub hash: Vec<u8>,
    /// Hash of the parent
    pub parent_hash: Vec<u8>,
    /// Hash of the uncles
    pub uncles_hash: Vec<u8>,
    /// Miner/author's address.
    pub author: ETHAddress,
    /// State root hash
    pub state_root: Vec<u8>,
    /// Transactions root hash
    pub transactions_root: Vec<u8>,
    /// Transactions receipts root hash
    pub receipts_root: Vec<u8>,
    /// Logs bloom
    pub logs_bloom: Vec<u8>,
    /// Difficulty
    pub difficulty: U256,
    /// Block number.
    pub number: u64,
    /// Gas Limit
    pub gas_limit: U256,
    /// Gas Used
    pub gas_used: U256,
    /// Timestamp
    pub timestamp: U256,
    /// Extra data
    pub extra_data: Vec<u8>,
}

impl<T> TryFrom<&Block<T>> for BlockHeader {
    type Error = anyhow::Error;

    fn try_from(value: &Block<T>) -> std::result::Result<Self, Self::Error> {
        let block_header = BlockHeader {
            hash: value
                .hash
                .ok_or_else(|| anyhow::format_err!("Unexpected pending block"))?
                .as_bytes()
                .to_vec(),
            parent_hash: value.parent_hash.as_bytes().to_vec(),
            uncles_hash: value.uncles_hash.as_bytes().to_vec(),
            author: value
                .author
                .ok_or_else(|| anyhow::format_err!("Unexpected pending block"))?
                .into(),
            state_root: value.state_root.as_bytes().to_vec(),
            transactions_root: value.transactions_root.as_bytes().to_vec(),
            receipts_root: value.receipts_root.as_bytes().to_vec(),
            logs_bloom: value.logs_bloom.map(|b| b.0.to_vec()).unwrap_or(vec![]),
            difficulty: eth_u256_to_move_u256(&value.difficulty),
            number: value
                .number
                .ok_or_else(|| anyhow::format_err!("Unexpected pending block"))?
                .as_u64(),
            gas_limit: eth_u256_to_move_u256(&value.gas_limit),
            gas_used: eth_u256_to_move_u256(&value.gas_used),
            timestamp: eth_u256_to_move_u256(&value.timestamp),
            extra_data: value.extra_data.to_vec(),
        };
        Ok(block_header)
    }
}

/// Rust bindings for RoochFramework session_key module
pub struct EthereumLightClientModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> EthereumLightClientModule<'a> {
    pub const GET_BLOCK_FUNCTION_NAME: &'static IdentStr = ident_str!("get_block");
    pub const SUBMIT_NEW_BLOCK_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("submit_new_block");

    pub fn get_block(&self, block_number: u64) -> Result<BlockHeader> {
        let call = FunctionCall::new(
            Self::function_id(Self::GET_BLOCK_FUNCTION_NAME),
            vec![],
            vec![MoveValue::U64(block_number).simple_serialize().unwrap()],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let block_header =
            self.caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    bcs::from_bytes::<BlockHeader>(&value.value)
                        .expect("should be a valid BlockHeader")
                })?;
        Ok(block_header)
    }

    pub fn create_submit_new_block_call(block_header: &BlockHeader) -> FunctionCall {
        Self::create_function_call(
            Self::SUBMIT_NEW_BLOCK_ENTRY_FUNCTION_NAME,
            vec![],
            vec![MoveValue::vector_u8(
                bcs::to_bytes(&block_header).expect("Serialize BlockHeader should success."),
            )],
        )
    }
}

impl<'a> ModuleBinding<'a> for EthereumLightClientModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = ROOCH_FRAMEWORK_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}

//TODO implement From<PrimitiveU256> for U256 in move side
pub fn eth_u256_to_move_u256(value: &ethers::types::U256) -> U256 {
    let mut bytes = [0u8; U256_NUM_BYTES];
    value.to_little_endian(&mut bytes);
    U256::from_le_bytes(&bytes)
}
