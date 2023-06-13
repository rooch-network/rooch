// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use ethers::types::Bytes;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use rooch_types::address::RoochAddress;

/// The Wallet API
/// This API is used to interact with the wallet in the Rooch node.
#[rpc(server, client, namespace = "wallet")]
pub trait WalletApi {
    #[method(name = "sign")]
    async fn sign(&self, address: RoochAddress, message: Bytes) -> RpcResult<Bytes>;

    /// Returns a list of addresses owned by the node.
    /// like `eth_accounts` in Ethereum
    #[method(name = "accounts")]
    async fn accounts(&self) -> RpcResult<Vec<RoochAddress>>;
}
