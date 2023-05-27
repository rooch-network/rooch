// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    api::{
        eth_api::{EthAPIServer, TransactionType},
        RoochRpcModule,
    },
    service::RpcService,
};
use ethers::types::{
    Block, BlockNumber, Bytes, OtherFields, Transaction, Withdrawal, transaction::eip2930::AccessList, H160, H256 as EtherH256, U256,
    U64
};

use jsonrpsee::{
    core::{async_trait, RpcResult},
    RpcModule,
};
use rooch_types::{
    transaction::{ethereum::EthereumTransaction, AbstractTransaction, TypedTransaction},
    H256,
};

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
    async fn get_chain_id(&self) -> RpcResult<String> {
        Ok(format!("0x{:X}", 10001))
    }

    async fn get_block_number(&self) -> RpcResult<String> {
        Ok(format!("0x{:X}", 100))
    }

    async fn get_block_by_number(
        &self,
        num: BlockNumber,
        include_txs: bool,
    ) -> RpcResult<Block<TransactionType>> {
        let block_number = num.as_number().unwrap();
        let parent_hash = EtherH256::zero();
        let gas_limit = U256::from(10_000_000);
        let gas_used = U256::from(5_000_000);

        let txs = if include_txs {
            vec![TransactionType::Full(Transaction {
                hash: EtherH256::zero(),
                nonce: U256::zero(),
                block_hash: Some(H256::zero()),
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
                v: U64::zero(),
                transaction_type: Default::default(),
                access_list: Some(AccessList::default()),
                max_priority_fee_per_gas: Default::default(),
                max_fee_per_gas: Default::default(),
                chain_id: Some(U256::from(10001)),
                other: OtherFields::default(),
            })]
        } else {
            vec![TransactionType::Hash(H256::zero())]
        };

        let block = Block {
            hash: Some(H256::zero()),
            parent_hash,
            uncles_hash: H256::zero(),
            author: Some(H160::zero()),
            state_root: H256::zero(),
            transactions_root: H256::zero(),
            receipts_root: H256::zero(),
            number: Some(block_number),
            gas_used,
            gas_limit,
            extra_data: Bytes::default(),
            logs_bloom: None,
            timestamp: U256::from(1_620_000_000),
            difficulty: U256::from(1_000_000),
            total_difficulty: Some(U256::from(10_000_000)),
            seal_fields: vec![],
            uncles: vec![],
            transactions: txs,
            size: None,
            mix_hash: None,
            nonce: None,
            base_fee_per_gas: Some(U256::zero()),
            withdrawals_root: Some(H256::zero()),
            withdrawals: Some(vec![Withdrawal::default()]),
            other: OtherFields::default(),
        };

        Ok(block)
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
