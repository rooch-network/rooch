// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    faucet_proxy::FaucetProxy, DiscordConfig, FaucetError, FaucetRequest, FaucetRequestWithInviter,
};
use move_core_types::u256::U256;
use rooch_rpc_api::jsonrpc_types::UnitedAddressView;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::{mpsc::Receiver, RwLock};

#[derive(Clone, Debug)]
pub struct App {
    pub faucet_proxy: FaucetProxy,
    pub err_receiver: Arc<RwLock<Receiver<FaucetError>>>,
    pub discord_config: DiscordConfig,
    pub is_loop_running: Arc<AtomicBool>,
}

impl App {
    pub fn new(
        faucet_proxy: FaucetProxy,
        err_receiver: Receiver<FaucetError>,
        discord_config: DiscordConfig,
    ) -> Self {
        Self {
            faucet_proxy,
            err_receiver: Arc::new(RwLock::new(err_receiver)),
            discord_config,
            is_loop_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn request(&self, request: FaucetRequest) -> Result<U256, FaucetError> {
        let amount = self
            .faucet_proxy
            .claim(request.claimer)
            .await
            .map_err(FaucetError::custom)?;
        Ok(amount)
    }

    pub async fn request_with_inviter(
        &self,
        request: FaucetRequestWithInviter,
    ) -> Result<U256, FaucetError> {
        let amount = self
            .faucet_proxy
            .claim_with_inviter(request.claimer, request.inviter, request.claimer_sign, request.public_key)
            .await
            .map_err(FaucetError::custom)?;
        Ok(amount)
    }

    pub async fn check_gas_balance(&self) -> Result<U256, FaucetError> {
        let balance = self
            .faucet_proxy
            .balance()
            .await
            .map_err(FaucetError::custom)?;
        Ok(balance)
    }

    pub async fn fetch_tweet(&self, tweet_id: String) -> Result<String, FaucetError> {
        let tweet = self
            .faucet_proxy
            .fetch_tweet(tweet_id)
            .await
            .map_err(FaucetError::custom)?;
        Ok(tweet.to_string())
    }

    pub async fn verify_and_binding_twitter_account(
        &self,
        tweet_id: String,
    ) -> Result<String, FaucetError> {
        let address = self
            .faucet_proxy
            .verify_and_binding_twitter_account(tweet_id)
            .await
            .map_err(FaucetError::custom)?;
        Ok(address.to_rooch_address().to_string())
    }
    pub async fn binding_twitter_account_with_inviter(
        &self,
        tweet_id: String,
        inviter: UnitedAddressView,
        claimer_sign: String,
        public_key: String
    ) -> Result<String, FaucetError> {
        let address = self
            .faucet_proxy
            .binding_twitter_account_with_inviter(tweet_id, inviter, claimer_sign, public_key)
            .await
            .map_err(FaucetError::custom)?;
        Ok(address.to_rooch_address().to_string())
    }
}
