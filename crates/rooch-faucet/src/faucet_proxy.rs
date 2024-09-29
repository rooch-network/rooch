// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{ClaimMessage, Faucet};
use anyhow::Result;
use coerce::actor::ActorRef;
use move_core_types::u256::U256;
use rooch_rpc_api::jsonrpc_types::UnitedAddressView;

#[derive(Debug, Clone)]
pub struct FaucetProxy {
    pub actor: ActorRef<Faucet>,
}

impl FaucetProxy {
    pub fn new(actor: ActorRef<Faucet>) -> Self {
        Self { actor }
    }

    pub async fn claim(&self, claimer: UnitedAddressView) -> Result<U256> {
        self.actor.send(ClaimMessage { claimer }).await?
    }

    pub async fn balance(&self) -> Result<U256> {
        self.actor.send(crate::BalanceMessage).await?
    }
}
