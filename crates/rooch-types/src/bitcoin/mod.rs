// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use self::types::Header;
use crate::{addresses::BITCOIN_MOVE_ADDRESS, into_address::IntoAddress};
use anyhow::Result;
use bitcoin::BlockHash;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, value::MoveValue,
};
use moveos_types::{
    module_binding::{ModuleBinding, MoveFunctionCaller},
    move_std::option::MoveOption,
    moveos_std::{
        object::{self, ObjectID},
        tx_context::TxContext,
    },
    state::MoveStructType,
    state::{MoveState, MoveStructState},
    transaction::FunctionCall,
};
use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &IdentStr = ident_str!("bitcoin");

/// Types mapping from Bitcoin Move types to Rust types
/// Module binding for the Framework
pub mod brc20;
pub mod genesis;
pub mod network;
pub mod ord;
pub mod types;
pub mod utxo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinBlockStore {
    pub latest_block_height: MoveOption<u64>,
    /// block hash -> block header table id
    pub blocks: ObjectID,
    /// block height -> block hash table id
    pub height_to_hash: ObjectID,
    /// block hash -> block height table id
    pub hash_to_height: ObjectID,
}

impl BitcoinBlockStore {
    pub fn object_id() -> ObjectID {
        object::named_object_id(&Self::struct_tag())
    }
}

impl MoveStructType for BitcoinBlockStore {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = ident_str!("BitcoinBlockStore");
    const ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;
}

impl MoveStructState for BitcoinBlockStore {
    fn struct_layout() -> move_core_types::value::MoveStructLayout {
        move_core_types::value::MoveStructLayout::new(vec![
            MoveOption::<u64>::type_layout(),
            ObjectID::type_layout(),
            ObjectID::type_layout(),
            ObjectID::type_layout(),
        ])
    }
}

/// Rust bindings for BitcoinMove bitcoin module
pub struct BitcoinModule<'a> {
    caller: &'a dyn MoveFunctionCaller,
}

impl<'a> BitcoinModule<'a> {
    pub const GET_BLOCK_FUNCTION_NAME: &'static IdentStr = ident_str!("get_block");
    pub const GET_BLOCK_BY_HEIGHT_FUNCTION_NAME: &'static IdentStr =
        ident_str!("get_block_by_height");
    pub const GET_BLOCK_HEIGHT_FUNCTION_NAME: &'static IdentStr = ident_str!("get_block_height");
    pub const GET_LATEST_BLOCK_HEIGHT_FUNCTION_NAME: &'static IdentStr =
        ident_str!("get_latest_block_height");
    pub const GET_UTXO_FUNCTION_NAME: &'static IdentStr = ident_str!("get_utxo");
    pub const SUBMIT_NEW_BLOCK_ENTRY_FUNCTION_NAME: &'static IdentStr =
        ident_str!("submit_new_block");
    pub const GET_GENESIS_BLOCK_HEIGHT_FUNCTION_NAME: &'static IdentStr =
        ident_str!("get_genesis_block_height");
    // pub const PROCESS_UTXOS_ENTRY_FUNCTION_NAME: &'static IdentStr = ident_str!("process_utxos");

    pub fn get_block(&self, block_hash: BlockHash) -> Result<Option<Header>> {
        let call = Self::create_function_call(
            Self::GET_BLOCK_FUNCTION_NAME,
            vec![],
            vec![MoveValue::Address(block_hash.into_address())],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let block_header =
            self.caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    bcs::from_bytes::<MoveOption<Header>>(&value.value)
                        .expect("should be a valid MoveOption<BlockHeader>")
                })?;
        Ok(block_header.into())
    }

    pub fn get_block_by_height(&self, block_height: u64) -> Result<Option<Header>> {
        let call = Self::create_function_call(
            Self::GET_BLOCK_BY_HEIGHT_FUNCTION_NAME,
            vec![],
            vec![MoveValue::U64(block_height)],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let block_header =
            self.caller
                .call_function(&ctx, call)?
                .into_result()
                .map(|mut values| {
                    let value = values.pop().expect("should have one return value");
                    bcs::from_bytes::<MoveOption<Header>>(&value.value)
                        .expect("should be a valid MoveOption<BlockHeader>")
                })?;
        Ok(block_header.into())
    }

    pub fn get_block_height(&self, block_hash: BlockHash) -> Result<Option<u64>> {
        let call = Self::create_function_call(
            Self::GET_BLOCK_HEIGHT_FUNCTION_NAME,
            vec![],
            vec![MoveValue::Address(block_hash.into_address())],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let height = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<MoveOption<u64>>(&value.value)
                    .expect("should be a valid MoveOption<u64>")
            })?;
        Ok(height.into())
    }

    pub fn get_latest_block_height(&self) -> Result<Option<u64>> {
        let call =
            Self::create_function_call(Self::GET_LATEST_BLOCK_HEIGHT_FUNCTION_NAME, vec![], vec![]);
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let height = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<MoveOption<u64>>(&value.value)
                    .expect("should be a valid MoveOption<u64>")
            })?;
        Ok(height.into())
    }

    pub fn get_genesis_block_height(&self) -> Result<u64> {
        let call = Self::create_function_call(
            Self::GET_GENESIS_BLOCK_HEIGHT_FUNCTION_NAME,
            vec![],
            vec![],
        );
        let ctx = TxContext::new_readonly_ctx(AccountAddress::ZERO);
        let height = self
            .caller
            .call_function(&ctx, call)?
            .into_result()
            .map(|mut values| {
                let value = values.pop().expect("should have one return value");
                bcs::from_bytes::<u64>(&value.value).expect("should be a valid u64")
            })?;
        Ok(height)
    }

    pub fn create_submit_new_block_call(block_height: u64, block: bitcoin::Block) -> FunctionCall {
        let block_hash = block.block_hash();
        let block = crate::bitcoin::types::Block::from(block);
        Self::create_function_call(
            Self::SUBMIT_NEW_BLOCK_ENTRY_FUNCTION_NAME,
            vec![],
            vec![
                MoveValue::U64(block_height),
                MoveValue::Address(block_hash.into_address()),
                MoveValue::vector_u8(
                    bcs::to_bytes(&block).expect("Serialize BlockHeader should success."),
                ),
            ],
        )
    }

    pub fn create_submit_new_block_call_bytes(
        block_height: u64,
        block_hash: Vec<u8>,
        block_body: Vec<u8>,
    ) -> Result<FunctionCall> {
        let block_hash = AccountAddress::from_bytes(block_hash)?;
        Ok(Self::create_function_call(
            Self::SUBMIT_NEW_BLOCK_ENTRY_FUNCTION_NAME,
            vec![],
            vec![
                MoveValue::U64(block_height),
                MoveValue::Address(block_hash),
                MoveValue::vector_u8(block_body),
            ],
        ))
    }
}

impl<'a> ModuleBinding<'a> for BitcoinModule<'a> {
    const MODULE_NAME: &'static IdentStr = MODULE_NAME;
    const MODULE_ADDRESS: AccountAddress = BITCOIN_MOVE_ADDRESS;

    fn new(caller: &'a impl MoveFunctionCaller) -> Self
    where
        Self: Sized,
    {
        Self { caller }
    }
}
