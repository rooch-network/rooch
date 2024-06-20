// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    bitcoin_relayer::BitcoinRelayer,
    ethereum_relayer::EthereumRelayer,
    messages::{GetReadyL1BlockMessage, GetReadyL1TxsMessage, SyncTick},
};
use anyhow::Result;
use coerce::actor::{ActorRef, ActorRefErr};
use rooch_types::transaction::{L1BlockWithBody, L1Transaction};

#[derive(Clone)]
pub enum RelayerProxy {
    Bitcoin(ActorRef<BitcoinRelayer>),
    Ethereum(ActorRef<EthereumRelayer>),
}

impl RelayerProxy {
    pub fn bitcoin(actor: ActorRef<BitcoinRelayer>) -> Self {
        Self::Bitcoin(actor)
    }
    pub fn ethereum(actor: ActorRef<EthereumRelayer>) -> Self {
        Self::Ethereum(actor)
    }

    pub fn name(&self) -> String {
        match self {
            Self::Bitcoin(actor) => actor.actor_id().to_string(),
            Self::Ethereum(actor) => actor.actor_id().to_string(),
        }
    }
    pub async fn sync(&self) -> Result<(), ActorRefErr> {
        match self {
            Self::Bitcoin(actor) => actor.notify(SyncTick {}).await,
            Self::Ethereum(actor) => actor.notify(SyncTick {}).await,
        }
    }

    pub async fn get_ready_l1_block(&self) -> Result<Option<L1BlockWithBody>> {
        match self {
            Self::Bitcoin(actor) => actor.send(GetReadyL1BlockMessage {}).await?,
            Self::Ethereum(actor) => actor.send(GetReadyL1BlockMessage {}).await?,
        }
    }

    pub async fn get_ready_l1_txs(&self) -> Result<Vec<L1Transaction>> {
        match self {
            Self::Bitcoin(actor) => actor.send(GetReadyL1TxsMessage {}).await?,
            Self::Ethereum(actor) => actor.send(GetReadyL1TxsMessage {}).await?,
        }
    }
}
