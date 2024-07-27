// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::btc::ord::InscriptionFilterView;
use crate::jsonrpc_types::btc::utxo::UTXOFilterView;
use crate::jsonrpc_types::{IndexerStateIDView, InscriptionPageView, StrView, UTXOPageView, BytesView, H256View};
use crate::RpcResult;
use jsonrpsee::proc_macros::rpc;
use rooch_open_rpc_macros::open_rpc;

#[open_rpc(namespace = "btc")]
#[rpc(server, client, namespace = "btc")]
#[async_trait]
pub trait BtcAPI {
    /// Query the UTXO via global index by UTXO filter
    #[method(name = "queryUTXOs")]
    async fn query_utxos(
        &self,
        filter: UTXOFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateIDView>,
        limit: Option<StrView<u64>>,
        descending_order: Option<bool>,
    ) -> RpcResult<UTXOPageView>;

    /// Query the Inscription via global index by Inscription filter
    #[method(name = "queryInscriptions")]
    async fn query_inscriptions(
        &self,
        filter: InscriptionFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateIDView>,
        limit: Option<StrView<u64>>,
        descending_order: Option<bool>,
    ) -> RpcResult<InscriptionPageView>;

    /// Broadcast a Bitcoin transaction
    #[method(name = "broadcastTX")]
    async fn broadcast_tx(
        &self,
        hex: BytesView,
        maxfeerate: Option<f64>,
        maxburnamount: Option<u64>,
    ) -> RpcResult<H256View>;
}
