// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::eth::{CallRequest, EthFeeHistory};
use crate::{
    api::{
        eth_api::{EthAPIServer, TransactionType},
        RoochRpcModule,
    },
    service::RpcService,
};
use ethers::types::{
    transaction::eip2930::AccessList, Block, BlockNumber, Bytes, OtherFields, Transaction,
    Withdrawal, H160, U256, U64,
};
use jsonrpsee::{
    core::{async_trait, Error as JsonRpcError, RpcResult},
    RpcModule,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rooch_types::{
    transaction::{ethereum::EthereumTransaction, AbstractTransaction, TypedTransaction},
    H256,
};
use std::iter;
use std::str::FromStr;
use std::time::SystemTime;

pub struct EthServer {
    rpc_service: RpcService,
}

impl EthServer {
    pub fn new(rpc_service: RpcService) -> Self {
        Self { rpc_service }
    }
}

#[async_trait]
impl EthAPIServer for EthServer {
    async fn net_version(&self) -> RpcResult<String> {
        Ok(String::from("1"))
    }

    async fn get_chain_id(&self) -> RpcResult<String> {
        Ok(format!("0x{:X}", 10001))
    }

    async fn get_block_number(&self) -> RpcResult<String> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");

        let block_number = now.as_secs();
        Ok(format!("0x{:X}", block_number))
    }

    async fn get_block_by_number(
        &self,
        num: BlockNumber,
        include_txs: bool,
    ) -> RpcResult<Block<TransactionType>> {
        let block_number = num.as_number().unwrap();
        let parent_hash = "0xe5ece23ec875db0657f964cbc74fa34439eef3ab3dc8664e7f4ae8b5c5c963e1"
            .parse()
            .unwrap();
        let gas_limit = U256::from_str("0x1c9c380").unwrap();
        let gas_used = U256::from_str("0xf4954d").unwrap();

        let txs = if include_txs {
            vec![TransactionType::Full(Transaction {
                hash: "0x96c133e6ee7966ee28e6a3b4abd38d1feb15bfcb9e3a36257bd4818ad679c26e"
                    .parse()
                    .unwrap(),
                nonce: U256::zero(),
                block_hash: Some(parent_hash),
                block_number: Some(block_number),
                transaction_index: Some(U64::from(0)),
                from: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e"
                    .parse()
                    .unwrap(),
                to: Some(
                    "0x832daF8DDe81fA5186EF2D04b3099251c508D5A1"
                        .parse()
                        .unwrap(),
                ),
                value: U256::from(1_000_000),
                gas_price: Some(U256::from(20_000_000_000u64)),
                gas: U256::from(21_000),
                input: vec![].into(),
                r: U256::zero(),
                s: U256::zero(),
                v: U64::one(),
                transaction_type: Default::default(),
                access_list: Some(AccessList::default()),
                max_priority_fee_per_gas: Default::default(),
                max_fee_per_gas: Default::default(),
                chain_id: Some(U256::from(10001)),
                other: OtherFields::default(),
            })]
        } else {
            vec![TransactionType::Hash(
                "0x96c133e6ee7966ee28e6a3b4abd38d1feb15bfcb9e3a36257bd4818ad679c26e"
                    .parse()
                    .unwrap(),
            )]
        };

        let block = Block {
            hash: Some(
                "0xa4161cc321054df6e370776f19a958950ce4237fca4aff57605efdcdd3b802f4"
                    .parse()
                    .unwrap(),
            ),
            parent_hash,
            uncles_hash: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347"
                .parse()
                .unwrap(),
            author: Some(
                "0xbaf6dc2e647aeb6f510f9e318856a1bcd66c5e19"
                    .parse()
                    .unwrap(),
            ),
            state_root: "0xde1cdf9816313c105a75eaaedab04815b1b7aa5650bf91b69749d71a36497243"
                .parse()
                .unwrap(),
            transactions_root: "0xdc8c2a8825fbbe669360d351e34f3ad09d320db83539c98e92bb18ea5fa93773"
                .parse()
                .unwrap(),
            receipts_root: "0x31814320e99d27d63448b25b122870e70427d8261bbaa3674e96dd686bcb507a"
                .parse()
                .unwrap(),
            number: Some(block_number),
            gas_used,
            gas_limit,
            extra_data: Bytes::from_str(
                "0x4d616465206f6e20746865206d6f6f6e20627920426c6f636b6e6174697665",
            )
            .unwrap(),
            logs_bloom: None,
            timestamp: U256::from_str("0x64731653").unwrap(),
            difficulty: U256::zero(),
            total_difficulty: Some(U256::from_str("0xc70d815d562d3cfa955").unwrap()),
            seal_fields: vec![],
            uncles: vec![],
            transactions: txs,
            size: None,
            mix_hash: None,
            nonce: None,
            base_fee_per_gas: Some(U256::from_str("0x52e0ce91c").unwrap()),
            withdrawals_root: Some(
                "0xdc8c2a8825fbbe669360d351e34f3ad09d320db83539c98e92bb18ea5fa93773"
                    .parse()
                    .unwrap(),
            ),
            withdrawals: Some(vec![Withdrawal {
                address: "0xb9d7934878b5fb9610b3fe8a5e441e8fad7e293f"
                    .parse()
                    .unwrap(),
                amount: U256::from_str("0xc7a3fa").unwrap(),
                index: U64::from_str("0x4e81dc").unwrap(),
                validator_index: U64::from_str("0x5be41").unwrap(),
            }]),
            other: OtherFields::default(),
        };

        Ok(block)
    }

    async fn get_balance(&self, _address: H160, _num: Option<BlockNumber>) -> RpcResult<U256> {
        Ok(U256::from(100) * U256::from(10_u64.pow(18)))
    }

    async fn estimate_gas(
        &self,
        _request: CallRequest,
        _num: Option<BlockNumber>,
    ) -> RpcResult<U256> {
        Ok(U256::from(10_000_000))
    }

    async fn fee_history(
        &self,
        block_count: U256,
        newest_block: BlockNumber,
        reward_percentiles: Option<Vec<f64>>,
    ) -> RpcResult<EthFeeHistory> {
        let mut rng = rand::thread_rng();

        let base_fee_per_gas: Vec<U256> = iter::repeat_with(|| {
            let random_value = rng.gen_range(1..100);
            U256::from(random_value)
        })
        .take(block_count.as_usize())
        .collect();

        let gas_used_ratio: Vec<f64> = iter::repeat_with(|| rng.gen_range(0.0..1.0))
            .take(block_count.as_usize())
            .collect();

        let reward = match reward_percentiles {
            Some(percentiles) => {
                let rewards: Vec<Vec<U256>> = (0..block_count.as_usize())
                    .map(|_| {
                        percentiles
                            .iter()
                            .map(|_| {
                                let random_value = rng.gen_range(1..100);
                                U256::from(random_value)
                            })
                            .collect()
                    })
                    .collect();
                Some(rewards)
            }
            None => None,
        };

        match newest_block.as_number() {
            Some(newest_block_num) => {
                let oldest_block_num = newest_block_num - block_count.low_u64();
                Ok(EthFeeHistory {
                    oldest_block: BlockNumber::Number(oldest_block_num),
                    base_fee_per_gas,
                    gas_used_ratio,
                    reward,
                })
            }
            None => {
                return Err(JsonRpcError::Custom(String::from(
                    "newest_block not a number",
                )))
            }
        }
    }

    async fn gas_price(&self) -> RpcResult<U256> {
        Ok(U256::from(20 * (10_u64.pow(9))))
    }

    async fn transaction_count(
        &self,
        _address: H160,
        _num: Option<BlockNumber>,
    ) -> RpcResult<U256> {
        let mut rng = StdRng::from_entropy();
        Ok(U256::from(rng.gen_range(0..10000)))
    }

    async fn send_raw_transaction(&self, bytes: Bytes) -> RpcResult<H256> {
        let tx = TypedTransaction::Ethereum(EthereumTransaction::decode(&bytes)?);
        let hash = tx.hash();
        let _output = self.rpc_service.execute_tx(tx).await?;
        Ok(hash)
    }
}

impl RoochRpcModule for EthServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
