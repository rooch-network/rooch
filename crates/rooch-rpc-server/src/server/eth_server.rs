// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::{aggregate_service::AggregateService, rpc_service::RpcService};
use ethers::types::{H160, U256, U64};
use jsonrpsee::{
    core::{async_trait, Error as JsonRpcError, RpcResult},
    RpcModule,
};
use moveos_types::{
    access_path::AccessPath, gas_config::GasConfig, h256::H256, state::MoveStructType,
};
use rooch_rpc_api::jsonrpc_types::eth::ethereum_types::bloom::Bloom;
use rooch_rpc_api::{
    api::{
        eth_api::{EthAPIServer, TransactionType},
        RoochRpcModule,
    },
    jsonrpc_types::{
        bytes::Bytes,
        eth::{
            ethereum_types::{
                block::{Block, BlockNumber},
                other_fields::OtherFields,
                withdrawal::Withdrawal,
            },
            AccessList, CallRequest, EthFeeHistory, Transaction, TransactionReceipt,
        },
        H160View, H256View, U256View, U64View,
    },
};
use rooch_types::{
    account::Account,
    address::{EthereumAddress, MultiChainAddress},
    framework::gas_coin::GasCoin,
    transaction::{AbstractTransaction, TypedTransaction},
};
use rooch_types::{chain_id::ChainID, transaction::ethereum::EthereumTransactionData};
use std::iter;
use std::str::FromStr;
use std::time::SystemTime;
use tracing::info;

pub struct EthServer {
    chain_id: ChainID,
    rpc_service: RpcService,
    aggregate_service: AggregateService,
}

impl EthServer {
    pub fn new(
        chain_id: ChainID,
        rpc_service: RpcService,
        aggregate_service: AggregateService,
    ) -> Self {
        Self {
            chain_id,
            rpc_service,
            aggregate_service,
        }
    }
}

#[async_trait]
impl EthAPIServer for EthServer {
    async fn net_version(&self) -> RpcResult<String> {
        Ok(String::from("1"))
    }

    async fn eth_chain_id(&self) -> RpcResult<String> {
        Ok(format!("0x{:x}", self.chain_id.id()))
    }

    async fn get_block_number(&self) -> RpcResult<String> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");

        let block_number = now.as_secs();
        Ok(format!("0x{:x}", block_number))
    }

    async fn get_block_by_number(
        &self,
        num: BlockNumber,
        include_txs: bool,
    ) -> RpcResult<Block<TransactionType>> {
        let block_number = num.as_number().unwrap();
        let parent_hash =
            H256::from_str("0xe5ece23ec875db0657f964cbc74fa34439eef3ab3dc8664e7f4ae8b5c5c963e1")
                .unwrap();
        let gas_limit = U256View::from(U256::from_str("0x1c9c380").unwrap());
        let gas_used = U256View::from(U256::from_str("0xf4954d").unwrap());

        let txs = if include_txs {
            vec![TransactionType::Full(Transaction {
                hash: H256View::from(
                    H256::from_str(
                        "0x96c133e6ee7966ee28e6a3b4abd38d1feb15bfcb9e3a36257bd4818ad679c26e",
                    )
                    .unwrap(),
                ),
                nonce: U256View::from(U256::zero()),
                block_hash: Some(parent_hash.into()),
                block_number: Some(block_number),
                transaction_index: Some(U64View::from(U64::from(0))),
                from: H160View::from(
                    H160::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap(),
                ),
                to: Some(H160View::from(
                    H160::from_str("0x832daF8DDe81fA5186EF2D04b3099251c508D5A1").unwrap(),
                )),
                value: U256View::from(U256::from(1_000_000)),
                gas_price: Some(U256View::from(U256::from(20_000_000_000u64))),
                gas: U256View::from(U256::from(21_000)),
                input: Bytes::new(vec![]),
                r: U256View::from(U256::zero()),
                s: U256View::from(U256::zero()),
                v: U64View::from(U64::one()),
                transaction_type: Default::default(),
                access_list: Some(AccessList::default()),
                max_priority_fee_per_gas: Default::default(),
                max_fee_per_gas: Default::default(),
                chain_id: Some(U256View::from(U256::from(10001))),
                other: OtherFields::default(),
            })]
        } else {
            vec![TransactionType::Hash(H256View::from(
                H256::from_str(
                    "0x96c133e6ee7966ee28e6a3b4abd38d1feb15bfcb9e3a36257bd4818ad679c26e",
                )
                .unwrap(),
            ))]
        };

        let block = Block {
            hash: Some(H256View::from(
                H256::from_str(
                    "0xa4161cc321054df6e370776f19a958950ce4237fca4aff57605efdcdd3b802f4",
                )
                .unwrap(),
            )),
            parent_hash: parent_hash.into(),
            uncles_hash: H256View::from(
                H256::from_str(
                    "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                )
                .unwrap(),
            ),
            author: Some(H160View::from(
                H160::from_str("0xbaf6dc2e647aeb6f510f9e318856a1bcd66c5e19").unwrap(),
            )),
            state_root: H256View::from(
                H256::from_str(
                    "0xde1cdf9816313c105a75eaaedab04815b1b7aa5650bf91b69749d71a36497243",
                )
                .unwrap(),
            ),
            transactions_root: H256View::from(
                H256::from_str(
                    "0xdc8c2a8825fbbe669360d351e34f3ad09d320db83539c98e92bb18ea5fa93773",
                )
                .unwrap(),
            ),
            receipts_root: H256View::from(
                H256::from_str(
                    "0x31814320e99d27d63448b25b122870e70427d8261bbaa3674e96dd686bcb507a",
                )
                .unwrap(),
            ),
            number: Some(block_number),
            gas_used,
            gas_limit,
            extra_data: Bytes::from_str(
                "0x4d616465206f6e20746865206d6f6f6e20627920426c6f636b6e6174697665",
            )
            .unwrap(),
            logs_bloom: None,
            timestamp: U256View::from(U256::from_str("0x64731653").unwrap()),
            difficulty: U256View::from(U256::zero()),
            total_difficulty: Some(U256View::from(
                U256::from_str("0xc70d815d562d3cfa955").unwrap(),
            )),
            seal_fields: vec![],
            uncles: vec![],
            transactions: txs,
            size: None,
            mix_hash: None,
            nonce: None,
            base_fee_per_gas: Some(U256View::from(U256::from_str("0x52e0ce91c").unwrap())),
            withdrawals_root: Some(H256View::from(
                H256::from_str(
                    "0xdc8c2a8825fbbe669360d351e34f3ad09d320db83539c98e92bb18ea5fa93773",
                )
                .unwrap(),
            )),
            withdrawals: Some(vec![Withdrawal {
                address: H160View::from(
                    H160::from_str("0xb9d7934878b5fb9610b3fe8a5e441e8fad7e293f").unwrap(),
                ),
                amount: U256View::from(U256::from_str("0xc7a3fa").unwrap()),
                index: U64View::from(U64::from_str("0x4e81dc").unwrap()),
                validator_index: U64View::from(U64::from_str("0x5be41").unwrap()),
            }]),
            other: OtherFields::default(),
        };

        Ok(block)
    }

    async fn get_balance(
        &self,
        address: H160View,
        _num: Option<BlockNumber>,
    ) -> RpcResult<U256View> {
        let account_address = self
            .rpc_service
            .resolve_address(MultiChainAddress::from(EthereumAddress(address.into())))
            .await?;
        //Return some balance if the account not exists.
        //Avoid MetaMask blocking the transaction submission.
        //TODO find a better way to solve this problem.
        let default_balance = GasCoin::scaling(100u64);
        let balance = self
            .aggregate_service
            .get_balances(account_address, Some(GasCoin::struct_tag()), None, 0)
            .await?
            .pop()
            .ok_or_else(|| JsonRpcError::Custom("Balance result must not empty".to_owned()))?
            .map(|(_cursor, balance_info)| {
                U256View::from(U256::from_little_endian(
                    &balance_info.balance.to_le_bytes(),
                ))
            })
            .unwrap_or(U256View::from(U256::from_little_endian(
                &default_balance.to_le_bytes(),
            )));
        Ok(balance)
    }

    async fn estimate_gas(
        &self,
        request: CallRequest,
        _num: Option<BlockNumber>,
    ) -> RpcResult<U256View> {
        let gas = match request.from {
            Some(from) => {
                let account_address = self
                    .rpc_service
                    .resolve_address(MultiChainAddress::from(EthereumAddress(from.into())))
                    .await?;
                let account_exists = self.rpc_service.exists_account(account_address).await?;
                if account_exists {
                    //TODO call dry run to estimate gas
                    U256View::from(U256::from(GasConfig::DEFAULT_MAX_GAS_AMOUNT))
                } else {
                    //The contract will automatically call faucet to deposit gas coin when the account does not exist.
                    //So, we return 0 gas to avoid MetaMask blocking the transaction submission.
                    //TODO when we implement the contract pay gas, we should return the real gas amount that user should pay.
                    U256View::from(U256::zero())
                }
            }
            None => U256View::from(U256::from(GasConfig::DEFAULT_MAX_GAS_AMOUNT)),
        };
        Ok(gas)
    }

    async fn fee_history(
        &self,
        block_count: U256View,
        newest_block: BlockNumber,
        reward_percentiles: Option<Vec<f64>>,
    ) -> RpcResult<EthFeeHistory> {
        let base_fee_per_gas: Vec<U256> = iter::repeat_with(U256::zero)
            .take(<U256View as Into<U256>>::into(block_count.clone()).as_usize())
            .collect();

        let gas_used_ratio: Vec<f64> = iter::repeat_with(|| 0.1)
            .take(<U256View as Into<U256>>::into(block_count.clone()).as_usize())
            .collect();

        let reward = match reward_percentiles {
            Some(percentiles) => {
                let rewards: Vec<Vec<U256>> =
                    (0..<U256View as Into<U256>>::into(block_count.clone()).as_usize())
                        .map(|_| percentiles.iter().map(|_| U256::from(1)).collect())
                        .collect();
                Some(rewards)
            }
            None => None,
        };

        match newest_block.as_number() {
            Some(newest_block_num) => {
                let oldest_block_num = <U64View as Into<U64>>::into(newest_block_num)
                    - <U256View as Into<U256>>::into(block_count).low_u64();
                let base_fee_per_gas_view: Vec<U256View> = base_fee_per_gas
                    .iter()
                    .map(|u256| U256View::from(*u256))
                    .collect();
                let reward_view: Option<Vec<Vec<U256View>>> = reward.map(|rewards| {
                    rewards
                        .iter()
                        .map(|inner| inner.iter().map(|u256| U256View::from(*u256)).collect())
                        .collect()
                });
                Ok(EthFeeHistory {
                    oldest_block: BlockNumber::Number(oldest_block_num.into()),
                    base_fee_per_gas: base_fee_per_gas_view,
                    gas_used_ratio,
                    reward: reward_view,
                })
            }
            None => {
                return Err(JsonRpcError::Custom(String::from(
                    "newest_block not a number",
                )))
            }
        }
    }

    async fn gas_price(&self) -> RpcResult<U256View> {
        //TODO read the get_gas_factor from contract.
        Ok(U256View::from(U256::from(1)))
    }

    async fn transaction_count(
        &self,
        address: H160View,
        _num: Option<BlockNumber>,
    ) -> RpcResult<U256View> {
        let account_address = self
            .rpc_service
            .resolve_address(MultiChainAddress::from(EthereumAddress(
                address.clone().into(),
            )))
            .await?;

        info!(
            "transaction_count source address: {:?}, rooch address: {:?}",
            address, account_address
        );

        let seq_number = self
            .rpc_service
            .get_states(AccessPath::resource(account_address, Account::struct_tag()))
            .await?
            .pop()
            .flatten()
            .map(|state_view| state_view.as_move_state::<Account>())
            .transpose()?
            .map_or(U256View::from(U256::zero()), |account| {
                U256View::from(<u64 as Into<U256>>::into(account.sequence_number))
            });

        info!("transaction_count seq_number: {:?}", seq_number);

        Ok(seq_number)
    }

    async fn send_raw_transaction(&self, bytes: Bytes) -> RpcResult<H256View> {
        info!("send_raw_transaction: {:?}", bytes);
        let eth_tx = EthereumTransactionData::decode(&bytes.0)?;
        info!("send_raw_transaction input: {:?}", eth_tx.0.input);
        let action = eth_tx.decode_calldata_to_action()?;
        info!(
            "send_raw_transaction decode_calldata_to_action: {:?}",
            action
        );
        info!(
            "send_raw_transaction from: {:?}, nonce: {:?}",
            eth_tx.0.from, eth_tx.0.nonce
        );

        let tx = TypedTransaction::Ethereum(eth_tx);
        info!(
            "send_raw_transaction authenticator_info: {:?}",
            tx.authenticator_info().unwrap()
        );

        let hash = H256View::from(tx.tx_hash());
        let _output = self.rpc_service.execute_tx(tx).await?;
        Ok(hash)
    }

    async fn transaction_receipt(&self, hash: H256View) -> RpcResult<Option<TransactionReceipt>> {
        let result = self
            .rpc_service
            .get_transaction_execution_infos_by_hash(vec![hash.into()])
            .await?
            .into_iter()
            .last()
            .and_then(|trans| {
                trans.map(|info| TransactionReceipt {
                    transaction_hash: info.tx_hash.into(),
                    block_hash: Some(info.state_root.into()),
                    block_number: Some(U64View::from(U64::from(10))),
                    gas_used: Some(U256View::from(<u64 as Into<U256>>::into(info.gas_used))),
                    status: Some(U64View::from(<u64 as Into<U64>>::into(
                        info.status.is_success() as u64,
                    ))),
                    cumulative_gas_used: U256View::from(<u64 as Into<U256>>::into(info.gas_used)),
                    contract_address: None,
                    logs: Vec::new(),
                    logs_bloom: Bloom::default(),
                    ..Default::default()
                })
            });

        Ok(result)
    }

    async fn transaction_by_hash(&self, hash: H256View) -> RpcResult<Option<Transaction>> {
        let resp = self
            .rpc_service
            .get_transaction_by_hash(hash.into())
            .await?
            .unwrap();

        // Create a new Transaction instance and populate its fields
        let transaction = Transaction {
            hash: resp.tx_hash().into(),
            nonce: U256View::from(U256::from(4391989)),
            block_hash: Some(H256View::from(H256::from_str("0xc2794a16acacd9f7670379ffd12b6968ff98e2a602f57d7d1f880220aa5a4973").unwrap())),
            block_number: Some(U64View::from(U64::from(8453214))),
            transaction_index: Some(U64View::from(U64::from(0))),
            from: EthereumAddress::try_from(resp.sender())?.0.into(),
            to: Some(H160View::from(H160::from_str("0x4200000000000000000000000000000000000015").unwrap())),
            value: U256View::from(U256::zero()),
            gas_price: Some(U256View::from(U256::zero())),
            gas: U256View::from(U256::from(1000000u64)),
            input: Bytes::new(
                hex::decode("015d8eb90000000000000000000000000000000000000000000000000000000000878c1c00000000000000000000000000000000000000000000000000000000644662bc0000000000000000000000000000000000000000000000000000001ee24fba17b7e19cc10812911dfa8a438e0a81a9933f843aa5b528899b8d9e221b649ae0df00000000000000000000000000000000000000000000000000000000000000060000000000000000000000007431310e026b69bfc676c0013e12a1a11411eec9000000000000000000000000000000000000000000000000000000000000083400000000000000000000000000000000000000000000000000000000000f4240").unwrap()
            ),
            r: U256View::from(U256::zero()),
            s: U256View::from(U256::zero()),
            v: U64View::from(U64::zero()),
            transaction_type: Some(U64View::from(U64::from(126))),
            access_list: None,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            chain_id: None,
            other: Default::default(),
        };

        Ok(Some(transaction))
    }

    async fn block_by_hash(
        &self,
        hash: H256View,
        include_txs: bool,
    ) -> RpcResult<Block<TransactionType>> {
        let block_number = U64View::from(U64::from(10));
        let parent_hash = H256View::from(
            H256::from_str("0xe5ece23ec875db0657f964cbc74fa34439eef3ab3dc8664e7f4ae8b5c5c963e1")
                .unwrap(),
        );
        let gas_limit = U256View::from(U256::from_str("0x1c9c380").unwrap());
        let gas_used = U256View::from(U256::from_str("0xf4954d").unwrap());

        let txs = if include_txs {
            vec![TransactionType::Full(Transaction {
                hash,
                nonce: U256View::from(U256::zero()),
                block_hash: Some(parent_hash.clone()),
                block_number: Some(block_number),
                transaction_index: Some(U64View::from(U64::from(0))),
                from: H160View::from(
                    H160::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap(),
                ),
                to: Some(H160View::from(
                    H160::from_str("0x832daF8DDe81fA5186EF2D04b3099251c508D5A1").unwrap(),
                )),
                value: U256View::from(U256::from(1_000_000)),
                gas_price: Some(U256View::from(U256::from(20_000_000_000u64))),
                gas: U256View::from(U256::from(21_000)),
                input: Bytes::new(vec![]),
                r: U256View::from(U256::zero()),
                s: U256View::from(U256::zero()),
                v: U64View::from(U64::one()),
                transaction_type: Default::default(),
                access_list: Some(AccessList::default()),
                max_priority_fee_per_gas: Default::default(),
                max_fee_per_gas: Default::default(),
                chain_id: Some(U256View::from(U256::from(10001))),
                other: OtherFields::default(),
            })]
        } else {
            vec![TransactionType::Hash(H256View::from(
                H256::from_str(
                    "0x96c133e6ee7966ee28e6a3b4abd38d1feb15bfcb9e3a36257bd4818ad679c26e",
                )
                .unwrap(),
            ))]
        };

        let block = Block {
            hash: Some(H256View::from(
                H256::from_str(
                    "0xa4161cc321054df6e370776f19a958950ce4237fca4aff57605efdcdd3b802f4",
                )
                .unwrap(),
            )),
            parent_hash,
            uncles_hash: H256View::from(
                H256::from_str(
                    "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                )
                .unwrap(),
            ),
            author: Some(H160View::from(
                H160::from_str("0xbaf6dc2e647aeb6f510f9e318856a1bcd66c5e19").unwrap(),
            )),
            state_root: H256View::from(
                H256::from_str(
                    "0xde1cdf9816313c105a75eaaedab04815b1b7aa5650bf91b69749d71a36497243",
                )
                .unwrap(),
            ),
            transactions_root: H256View::from(
                H256::from_str(
                    "0xdc8c2a8825fbbe669360d351e34f3ad09d320db83539c98e92bb18ea5fa93773",
                )
                .unwrap(),
            ),
            receipts_root: H256View::from(
                H256::from_str(
                    "0x31814320e99d27d63448b25b122870e70427d8261bbaa3674e96dd686bcb507a",
                )
                .unwrap(),
            ),
            number: Some(block_number),
            gas_used,
            gas_limit,
            extra_data: Bytes::from_str(
                "0x4d616465206f6e20746865206d6f6f6e20627920426c6f636b6e6174697665",
            )
            .unwrap(),
            logs_bloom: None,
            timestamp: U256View::from(U256::from_str("0x64731653").unwrap()),
            difficulty: U256View::from(U256::zero()),
            total_difficulty: Some(U256View::from(
                U256::from_str("0xc70d815d562d3cfa955").unwrap(),
            )),
            seal_fields: vec![],
            uncles: vec![],
            transactions: txs,
            size: None,
            mix_hash: None,
            nonce: None,
            base_fee_per_gas: Some(U256View::from(U256::from_str("0x52e0ce91c").unwrap())),
            withdrawals_root: Some(H256View::from(
                H256::from_str(
                    "0xdc8c2a8825fbbe669360d351e34f3ad09d320db83539c98e92bb18ea5fa93773",
                )
                .unwrap(),
            )),
            withdrawals: Some(vec![Withdrawal {
                address: H160View::from(
                    H160::from_str("0xb9d7934878b5fb9610b3fe8a5e441e8fad7e293f").unwrap(),
                ),
                amount: U256View::from(U256::from_str("0xc7a3fa").unwrap()),
                index: U64View::from(U64::from_str("0x4e81dc").unwrap()),
                validator_index: U64View::from(U64::from_str("0x5be41").unwrap()),
            }]),
            other: OtherFields::default(),
        };

        Ok(block)
    }
}

impl RoochRpcModule for EthServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
