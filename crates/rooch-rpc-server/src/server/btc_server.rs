// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::rpc_service::RpcService;
use anyhow::Result;
use bitcoincore_rpc::bitcoin::Txid;
use jsonrpsee::{core::async_trait, RpcModule};
use rooch_rpc_api::api::btc_api::BtcAPIServer;
use rooch_rpc_api::api::{RoochRpcModule, DEFAULT_RESULT_LIMIT_USIZE, MAX_RESULT_LIMIT_USIZE};
use rooch_rpc_api::jsonrpc_types::btc::ord::{InscriptionFilterView, InscriptionStateView};
use rooch_rpc_api::jsonrpc_types::btc::utxo::{UTXOFilterView, UTXOStateView};
use rooch_rpc_api::jsonrpc_types::{
    BytesView, IndexerStateIDView, InscriptionPageView, StrView, UTXOPageView,
};
use rooch_rpc_api::RpcResult;
use std::cmp::min;

pub struct BtcServer {
    rpc_service: RpcService,
}

impl BtcServer {
    pub async fn new(rpc_service: RpcService) -> Result<Self> {
        Ok(Self { rpc_service })
    }
}

#[async_trait]
impl BtcAPIServer for BtcServer {
    async fn query_utxos(
        &self,
        filter: UTXOFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateIDView>,
        limit: Option<StrView<u64>>,
        descending_order: Option<bool>,
    ) -> RpcResult<UTXOPageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let descending_order = descending_order.unwrap_or(true);

        let global_state_filter = UTXOFilterView::into_global_state_filter(filter)?;
        let object_states = self
            .rpc_service
            .query_object_states(
                global_state_filter,
                cursor.map(|c| c.into()),
                limit_of + 1,
                descending_order,
                false,
                false,
            )
            .await?;

        let mut data = object_states
            .into_iter()
            .map(UTXOStateView::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);
        let next_cursor = data.last().cloned().map_or(cursor, |t| Some(t.indexer_id));

        Ok(UTXOPageView {
            data,
            next_cursor,
            has_next_page,
        })
    }

    async fn query_inscriptions(
        &self,
        filter: InscriptionFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateIDView>,
        limit: Option<StrView<u64>>,
        descending_order: Option<bool>,
    ) -> RpcResult<InscriptionPageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let descending_order = descending_order.unwrap_or(true);

        let global_state_filter = InscriptionFilterView::into_global_state_filter(filter)?;
        let object_states = self
            .rpc_service
            .query_object_states(
                global_state_filter,
                cursor.map(Into::into),
                limit_of + 1,
                descending_order,
                false,
                false,
            )
            .await?;

        let mut data = object_states
            .into_iter()
            .map(InscriptionStateView::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);
        let next_cursor = data.last().cloned().map_or(cursor, |t| Some(t.indexer_id));

        Ok(InscriptionPageView {
            data,
            next_cursor,
            has_next_page,
        })
    }

    async fn broadcast_tx(
        &self,
        hex: BytesView,
        maxfeerate: Option<f64>,
        maxburnamount: Option<f64>,
    ) -> RpcResult<String> {
        let tx_hex = hex::encode(hex.0);
        let txid: Txid = self
            .rpc_service
            .broadcast_bitcoin_transaction(tx_hex, maxfeerate, maxburnamount)
            .await?;

        Ok(txid.to_string())
    }
}

impl RoochRpcModule for BtcServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
