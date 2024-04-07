// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::{aggregate_service::AggregateService, rpc_service::RpcService};
use anyhow::Result;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    RpcModule,
};
use move_core_types::account_address::AccountAddress;
use rooch_rpc_api::api::btc_api::BtcAPIServer;
use rooch_rpc_api::api::{RoochRpcModule, DEFAULT_RESULT_LIMIT_USIZE, MAX_RESULT_LIMIT_USIZE};
use rooch_rpc_api::jsonrpc_types::btc::ord::{InscriptionFilterView, InscriptionStateView};
use rooch_rpc_api::jsonrpc_types::btc::utxo::{UTXOFilterView, UTXOStateView};
use rooch_rpc_api::jsonrpc_types::{InscriptionPageView, StrView, UTXOPageView};
use rooch_types::address::MultiChainAddress;
use rooch_types::indexer::state::IndexerStateID;
use rooch_types::multichain_id::RoochMultiChainID;
use std::cmp::min;

pub struct BtcServer {
    rpc_service: RpcService,
    aggregate_service: AggregateService,
    btc_network: u8,
}

impl BtcServer {
    pub async fn new(rpc_service: RpcService, aggregate_service: AggregateService) -> Result<Self> {
        let btc_network = rpc_service.get_bitcoin_network().await?;
        Ok(Self {
            rpc_service,
            aggregate_service,
            btc_network,
        })
    }
}

#[async_trait]
impl BtcAPIServer for BtcServer {
    async fn query_utxos(
        &self,
        filter: UTXOFilterView,
        // exclusive cursor if `Some`, otherwise start from the beginning
        cursor: Option<IndexerStateID>,
        limit: Option<StrView<usize>>,
        descending_order: Option<bool>,
    ) -> RpcResult<UTXOPageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let descending_order = descending_order.unwrap_or(true);

        let resolve_address = match filter.clone() {
            UTXOFilterView::Owner(address) => {
                let multi_chain_address = MultiChainAddress::try_from_str_with_multichain_id(
                    RoochMultiChainID::Bitcoin,
                    address.to_string().as_str(),
                )?;
                self.rpc_service
                    .resolve_address(multi_chain_address)
                    .await?
            }
            _ => AccountAddress::ZERO,
        };

        let global_state_filter =
            UTXOFilterView::into_global_state_filter(filter, resolve_address)?;
        let states = self
            .rpc_service
            .query_object_states(global_state_filter, cursor, limit_of + 1, descending_order)
            .await?;

        let mut data = self
            .aggregate_service
            .pack_uxtos(states)
            .await?
            .into_iter()
            .map(|v| UTXOStateView::try_new_from_utxo_state(v, self.btc_network))
            .collect::<Result<Vec<_>, _>>()?;

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);
        let next_cursor = data.last().cloned().map_or(cursor, |t| {
            Some(IndexerStateID::new(t.tx_order, t.state_index))
        });

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
        cursor: Option<IndexerStateID>,
        limit: Option<StrView<usize>>,
        descending_order: Option<bool>,
    ) -> RpcResult<InscriptionPageView> {
        let limit_of = min(
            limit.map(Into::into).unwrap_or(DEFAULT_RESULT_LIMIT_USIZE),
            MAX_RESULT_LIMIT_USIZE,
        );
        let descending_order = descending_order.unwrap_or(true);

        let resolve_address = match filter.clone() {
            InscriptionFilterView::Owner(address) => {
                let multi_chain_address = MultiChainAddress::try_from_str_with_multichain_id(
                    RoochMultiChainID::Bitcoin,
                    address.to_string().as_str(),
                )?;
                self.rpc_service
                    .resolve_address(multi_chain_address)
                    .await?
            }
            _ => AccountAddress::ZERO,
        };

        let global_state_filter =
            InscriptionFilterView::into_global_state_filter(filter, resolve_address)?;
        let states = self
            .rpc_service
            .query_object_states(global_state_filter, cursor, limit_of + 1, descending_order)
            .await?;

        let mut data = self
            .aggregate_service
            .pack_inscriptions(states)
            .await?
            .into_iter()
            .map(|v| InscriptionStateView::try_new_from_inscription_state(v, self.btc_network))
            .collect::<Result<Vec<_>, _>>()?;

        let has_next_page = data.len() > limit_of;
        data.truncate(limit_of);
        let next_cursor = data.last().cloned().map_or(cursor, |t| {
            Some(IndexerStateID::new(t.tx_order, t.state_index))
        });

        Ok(InscriptionPageView {
            data,
            next_cursor,
            has_next_page,
        })
    }
}

impl RoochRpcModule for BtcServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
