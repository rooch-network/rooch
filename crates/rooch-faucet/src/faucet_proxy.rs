// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{ClaimMessage, ClaimWithInviterMessage, Faucet};
use anyhow::Result;
use coerce::actor::ActorRef;
use move_core_types::u256::U256;
use moveos_types::moveos_std::object::ObjectID;
use rooch_rpc_api::jsonrpc_types::UnitedAddressView;
use rooch_types::address::BitcoinAddress;

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

    pub async fn claim_with_inviter(
        &self,
        claimer: UnitedAddressView,
        inviter: UnitedAddressView,
        claimer_sign: String,
        public_key: String,
        message: String,
    ) -> Result<U256> {
        self.actor
            .send(ClaimWithInviterMessage {
                claimer,
                inviter,
                claimer_sign,
                public_key,
                message,
            })
            .await?
    }

    pub async fn balance(&self) -> Result<U256> {
        self.actor.send(crate::BalanceMessage).await?
    }

    pub async fn fetch_tweet(&self, tweet_id: String) -> Result<ObjectID> {
        self.actor
            .send(crate::FetchTweetMessage { tweet_id })
            .await?
    }

    pub async fn verify_and_binding_twitter_account(
        &self,
        tweet_id: String,
    ) -> Result<BitcoinAddress> {
        self.actor
            .send(crate::VerifyAndBindingTwitterAccountMessage { tweet_id })
            .await?
    }

    pub async fn binding_twitter_account_with_inviter(
        &self,
        tweet_id: String,
        inviter: UnitedAddressView,
        claimer_sign: String,
        public_key: String,
        message: String,
    ) -> Result<BitcoinAddress> {
        self.actor
            .send(crate::BindingTwitterAccountMessageWithInviter {
                tweet_id,
                inviter,
                claimer_sign,
                public_key,
                message,
            })
            .await?
    }
}
