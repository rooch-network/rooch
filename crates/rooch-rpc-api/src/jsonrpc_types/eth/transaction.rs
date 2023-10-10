// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    ethereum_types::{bloom::Bloom, ens::NameOrAddress, log::Log, other_fields::OtherFields},
    AccessList,
};
use crate::jsonrpc_types::{bytes::Bytes, H256View, StrView};
use ethers::types::{H160, U256, U64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Details of a signed transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    /// The transaction's hash
    pub hash: H256View,

    /// The transaction's nonce
    pub nonce: StrView<U256>,

    /// Block hash. None when pending.
    #[serde(default, rename = "blockHash")]
    pub block_hash: Option<H256View>,

    /// Block number. None when pending.
    #[serde(default, rename = "blockNumber")]
    pub block_number: Option<StrView<U64>>,

    /// Transaction Index. None when pending.
    #[serde(default, rename = "transactionIndex")]
    pub transaction_index: Option<StrView<U64>>,

    /// Sender
    #[serde(default)]
    pub from: StrView<H160>,

    /// Recipient (None when contract creation)
    #[serde(default)]
    pub to: Option<StrView<H160>>,

    /// Transferred value
    pub value: StrView<U256>,

    /// Gas Price, null for Type 2 transactions
    #[serde(rename = "gasPrice")]
    pub gas_price: Option<StrView<U256>>,

    /// Gas amount
    pub gas: StrView<U256>,

    /// Input data
    pub input: Bytes,

    /// ECDSA recovery id
    pub v: StrView<U64>,

    /// ECDSA signature r
    pub r: StrView<U256>,

    /// ECDSA signature s
    pub s: StrView<U256>,

    ///////////////// Optimism-specific transaction fields //////////////
    /// The source-hash that uniquely identifies the origin of the deposit
    #[cfg(feature = "optimism")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "sourceHash"
    )]
    pub source_hash: Option<H256>,

    /// The ETH value to mint on L2
    #[cfg(feature = "optimism")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mint: Option<U256>,

    /// True if the transaction does not interact with the L2 block gas pool
    #[cfg(feature = "optimism")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "isSystemTx"
    )]
    pub is_system_tx: Option<bool>,

    /////////////////  Celo-specific transaction fields /////////////////
    /// The currency fees are paid in (None for native currency)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none", rename = "feeCurrency")]
    pub fee_currency: Option<Address>,

    /// Gateway fee recipient (None for no gateway fee paid)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "gatewayFeeRecipient"
    )]
    pub gateway_fee_recipient: Option<Address>,

    /// Gateway fee amount (None for no gateway fee paid)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none", rename = "gatewayFee")]
    pub gateway_fee: Option<U256>,

    // EIP2718
    /// Transaction type, Some(2) for EIP-1559 transaction,
    /// Some(1) for AccessList transaction, None for Legacy
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<StrView<U64>>,

    // EIP2930
    #[serde(
        rename = "accessList",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub access_list: Option<AccessList>,

    #[serde(
        rename = "maxPriorityFeePerGas",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    /// Represents the maximum tx fee that will go to the miner as part of the user's
    /// fee payment. It serves 3 purposes:
    /// 1. Compensates miners for the uncle/ommer risk + fixed costs of including transaction in a
    /// block; 2. Allows users with high opportunity costs to pay a premium to miners;
    /// 3. In times where demand exceeds the available block space (i.e. 100% full, 30mm gas),
    /// this component allows first price auctions (i.e. the pre-1559 fee model) to happen on the
    /// priority fee.
    ///
    /// More context [here](https://hackmd.io/@q8X_WM2nTfu6nuvAzqXiTQ/1559-wallets)
    pub max_priority_fee_per_gas: Option<StrView<U256>>,

    #[serde(
        rename = "maxFeePerGas",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    /// Represents the maximum amount that a user is willing to pay for their tx (inclusive of
    /// baseFeePerGas and maxPriorityFeePerGas). The difference between maxFeePerGas and
    /// baseFeePerGas + maxPriorityFeePerGas is “refunded” to the user.
    pub max_fee_per_gas: Option<StrView<U256>>,

    #[serde(rename = "chainId", default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<StrView<U256>>,

    /// Captures unknown fields such as additional fields used by L2s
    #[cfg(not(any(feature = "celo", feature = "optimism")))]
    #[serde(flatten)]
    pub other: OtherFields,
}

/// Parameters for sending a transaction
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    /// Sender address or ENS name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<StrView<H160>>,

    /// Recipient address (None for contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<NameOrAddress>,

    /// Supplied gas (None for sensible default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<StrView<U256>>,

    /// Gas price (None for sensible default)
    #[serde(rename = "gasPrice")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<StrView<U256>>,

    /// Transferred value (None for no transfer)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<StrView<U256>>,

    /// The compiled code of a contract OR the first 4 bytes of the hash of the
    /// invoked method signature and encoded parameters. For details see Ethereum Contract ABI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Bytes>,

    /// Transaction nonce (None for next available nonce)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<StrView<U256>>,

    /// Chain ID (None for mainnet)
    #[serde(skip_serializing)]
    #[serde(default, rename = "chainId")]
    pub chain_id: Option<StrView<U64>>,

    /////////////////  Celo-specific transaction fields /////////////////
    /// The currency fees are paid in (None for native currency)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_currency: Option<Address>,

    /// Gateway fee recipient (None for no gateway fee paid)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_fee_recipient: Option<Address>,

    /// Gateway fee amount (None for no gateway fee paid)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_fee: Option<U256>,
}

/// "Receipt" of an executed transaction: details of its execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    /// Transaction hash.
    #[serde(rename = "transactionHash")]
    pub transaction_hash: H256View,
    /// Index within the block.
    #[serde(rename = "transactionIndex")]
    pub transaction_index: StrView<U64>,
    /// Hash of the block this transaction was included within.
    #[serde(rename = "blockHash")]
    pub block_hash: Option<H256View>,
    /// Number of the block this transaction was included within.
    #[serde(rename = "blockNumber")]
    pub block_number: Option<StrView<U64>>,
    /// address of the sender.
    pub from: StrView<H160>,
    // address of the receiver. null when its a contract creation transaction.
    pub to: Option<StrView<H160>>,
    /// Cumulative gas used within the block after this was executed.
    #[serde(rename = "cumulativeGasUsed")]
    pub cumulative_gas_used: StrView<U256>,
    /// Gas used by this transaction alone.
    ///
    /// Gas used is `None` if the the client is running in light client mode.
    #[serde(rename = "gasUsed")]
    pub gas_used: Option<StrView<U256>>,
    /// Contract address created, or `None` if not a deployment.
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<StrView<H160>>,
    /// Logs generated within this transaction.
    pub logs: Vec<Log>,
    /// Status: either 1 (success) or 0 (failure). Only present after activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    pub status: Option<StrView<U64>>,
    /// State root. Only present before activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<H256View>,
    /// Logs bloom
    #[serde(rename = "logsBloom")]
    pub logs_bloom: Bloom,
    /// Transaction type, Some(1) for AccessList transaction, None for Legacy
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<StrView<U64>>,
    /// The price paid post-execution by the transaction (i.e. base fee + priority fee).
    /// Both fields in 1559-style transactions are *maximums* (max fee + max priority fee), the
    /// amount that's actually paid by users can only be determined post-execution
    #[serde(
        rename = "effectiveGasPrice",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub effective_gas_price: Option<StrView<U256>>,
    /// Captures unknown fields such as additional fields used by L2s
    #[cfg(not(feature = "celo"))]
    #[serde(flatten)]
    pub other: OtherFields,
}
