// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    faucet_module, invitation_module, tweet_fetcher_module, tweet_v2_module, twitter_account_module,
};
use crate::{metrics::FaucetMetrics, FaucetError};
use anyhow::{bail, Result};
use async_trait::async_trait;
use clap::Parser;
use coerce::actor::context::ActorContext;
use coerce::actor::message::{Handler, Message};
use coerce::actor::Actor;
use move_core_types::account_address::AccountAddress;
use move_core_types::u256::U256;
use move_core_types::vm_status::AbortLocation;
use moveos_types::moveos_std::object::ObjectID;
use moveos_types::transaction::{FunctionCall, MoveAction};
use prometheus::Registry;
use rooch_rpc_api::jsonrpc_types::btc::utxo::UTXOFilterView;
use rooch_rpc_api::jsonrpc_types::{KeptVMStatusView, UnitedAddressView, VMStatusView};
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_rpc_client::Client;
use rooch_types::address::{BitcoinAddress, ParsedAddress, RoochAddress};
use serde::de::DeserializeOwned;
use tokio::sync::mpsc::Sender;

#[derive(Parser, Debug, Clone)]
pub struct FaucetConfig {
    #[clap(long, default_value_t = 10000)]
    pub max_request_queue_length: u64,

    #[clap(long)]
    pub faucet_module_address: ParsedAddress,

    #[clap(long)]
    pub invitation_module_address: ParsedAddress,

    #[clap(long)]
    pub faucet_object_id: ObjectID,

    #[clap(long)]
    pub invitation_object_id: ObjectID,

    /// The address to send the faucet claim transaction
    /// Default is the active address in the wallet
    #[clap(long, default_value = "default")]
    pub faucet_sender: ParsedAddress,
}

pub struct Faucet {
    faucet_sender: RoochAddress,
    faucet_module_address: AccountAddress,
    invitation_module_address: AccountAddress,
    faucet_object_id: ObjectID,
    invitation_object_id: ObjectID,
    context: WalletContext,
    faucet_error_sender: Sender<FaucetError>,
    // metrics: FaucetMetrics,
}

pub struct ClaimMessage {
    pub claimer: UnitedAddressView,
}

impl Message for ClaimMessage {
    type Result = Result<U256>;
}

pub struct ClaimWithInviterMessage {
    pub claimer: UnitedAddressView,
    pub inviter: UnitedAddressView,
    pub claimer_sign: String,
    pub public_key: String,
    pub message: String,
}

impl Message for ClaimWithInviterMessage {
    type Result = Result<U256>;
}

pub struct BalanceMessage;

impl Message for BalanceMessage {
    type Result = Result<U256>;
}

#[async_trait]
impl Actor for Faucet {}

#[async_trait]
impl Handler<ClaimMessage> for Faucet {
    async fn handle(&mut self, msg: ClaimMessage, _ctx: &mut ActorContext) -> Result<U256> {
        self.claim(msg.claimer).await
    }
}

#[async_trait]
impl Handler<ClaimWithInviterMessage> for Faucet {
    async fn handle(
        &mut self,
        msg: ClaimWithInviterMessage,
        _ctx: &mut ActorContext,
    ) -> Result<U256> {
        self.claim_with_inviter(
            msg.claimer,
            msg.inviter,
            msg.claimer_sign,
            msg.public_key,
            msg.message,
        )
        .await
    }
}

#[async_trait]
impl Handler<BalanceMessage> for Faucet {
    async fn handle(&mut self, _msg: BalanceMessage, _ctx: &mut ActorContext) -> Result<U256> {
        self.balance().await
    }
}

pub struct FetchTweetMessage {
    pub tweet_id: String,
}

impl Message for FetchTweetMessage {
    type Result = Result<ObjectID>;
}

#[async_trait]
impl Handler<FetchTweetMessage> for Faucet {
    async fn handle(
        &mut self,
        msg: FetchTweetMessage,
        _ctx: &mut ActorContext,
    ) -> Result<ObjectID> {
        self.fetch_tweet(msg.tweet_id).await
    }
}

pub struct VerifyAndBindingTwitterAccountMessage {
    pub tweet_id: String,
}

impl Message for VerifyAndBindingTwitterAccountMessage {
    type Result = Result<BitcoinAddress>;
}

#[async_trait]
impl Handler<VerifyAndBindingTwitterAccountMessage> for Faucet {
    async fn handle(
        &mut self,
        msg: VerifyAndBindingTwitterAccountMessage,
        _ctx: &mut ActorContext,
    ) -> Result<BitcoinAddress> {
        self.verify_and_binding_twitter_account(msg.tweet_id).await
    }
}

pub struct BindingTwitterAccountMessageWithInviter {
    pub tweet_id: String,
    pub inviter: UnitedAddressView,
    pub claimer_sign: String,
    pub public_key: String,
    pub message: String,
}

impl Message for BindingTwitterAccountMessageWithInviter {
    type Result = Result<BitcoinAddress>;
}

#[async_trait]
impl Handler<BindingTwitterAccountMessageWithInviter> for Faucet {
    async fn handle(
        &mut self,
        msg: BindingTwitterAccountMessageWithInviter,
        _ctx: &mut ActorContext,
    ) -> Result<BitcoinAddress> {
        self.binding_twitter_account_with_inviter(
            msg.tweet_id,
            msg.inviter,
            msg.claimer_sign,
            msg.public_key,
            msg.message,
        )
        .await
    }
}

impl Faucet {
    pub fn new(
        prometheus_registry: &Registry,
        wallet_context: WalletContext,
        config: FaucetConfig,
        faucet_error_sender: Sender<FaucetError>,
    ) -> Result<Self> {
        let _metrics = FaucetMetrics::new(prometheus_registry);
        let faucet_module_address = wallet_context.resolve_address(config.faucet_module_address)?;
        let invitation_module_address =
            wallet_context.resolve_address(config.invitation_module_address)?;
        let faucet_sender = wallet_context.resolve_address(config.faucet_sender)?;
        Ok(Self {
            faucet_sender: faucet_sender.into(),
            faucet_module_address,
            invitation_module_address,
            faucet_object_id: config.faucet_object_id,
            invitation_object_id: config.invitation_object_id,
            context: wallet_context,
            faucet_error_sender,
        })
    }

    async fn claim(&mut self, claimer: UnitedAddressView) -> Result<U256> {
        tracing::debug!("claim address: {}", claimer);
        let claimer_addr: AccountAddress = claimer.clone().into();

        let client = self.context.get_client().await?;
        let utxo_ids = Self::get_utxos(&client, claimer.clone()).await?;
        let claim_amount = Self::check_claim(
            &client,
            self.faucet_module_address,
            self.faucet_object_id.clone(),
            claimer_addr,
            utxo_ids.clone(),
        )
        .await?;

        let function_call = faucet_module::claim_function_call(
            self.faucet_module_address,
            self.faucet_object_id.clone(),
            claimer_addr,
            utxo_ids,
        );
        let action = MoveAction::Function(function_call);
        let tx_data = self
            .context
            .build_tx_data(self.faucet_sender, action, None)
            .await?;
        let response = self
            .context
            .sign_and_execute(self.faucet_sender, tx_data)
            .await?;
        match response.execution_info.status {
            KeptVMStatusView::Executed => {
                tracing::info!("Claim success for {}", claimer);
                Ok(claim_amount)
            }
            status => {
                let err = FaucetError::Transfer(format!("{:?}", status));
                if let Err(e) = self.faucet_error_sender.try_send(err) {
                    tracing::warn!("Failed to send error to faucet_error_sender: {:?}", e);
                }
                bail!("Claim failed, Unexpected VM status: {:?}", status)
            }
        }
    }

    async fn claim_with_inviter(
        &mut self,
        claimer: UnitedAddressView,
        inviter: UnitedAddressView,
        claimer_sign: String,
        public_key: String,
        message: String,
    ) -> Result<U256> {
        tracing::debug!("claim address: {}, inviter address: {}", claimer, inviter);
        let claimer_addr: AccountAddress = claimer.clone().into();
        let inviter_addr: AccountAddress = inviter.clone().into();
        let client = self.context.get_client().await?;
        let utxo_ids = Self::get_utxos(&client, claimer.clone()).await?;
        let claim_amount = Self::check_claim(
            &client,
            self.faucet_module_address,
            self.faucet_object_id.clone(),
            claimer_addr,
            utxo_ids.clone(),
        )
        .await?;

        let function_call = invitation_module::claim_from_faucet_function_call(
            self.invitation_module_address,
            self.faucet_object_id.clone(),
            self.invitation_object_id.clone(),
            claimer.to_string(),
            utxo_ids,
            inviter_addr,
            public_key,
            claimer_sign,
            message,
        );
        let action = MoveAction::Function(function_call);
        let tx_data = self
            .context
            .build_tx_data(self.faucet_sender, action, None)
            .await?;
        let response = self
            .context
            .sign_and_execute(self.faucet_sender, tx_data)
            .await?;
        match response.execution_info.status {
            KeptVMStatusView::Executed => {
                tracing::info!("Claim success for {}", claimer);
                Ok(claim_amount)
            }
            status => {
                let err = FaucetError::Transfer(format!("{:?}", status));
                if let Err(e) = self.faucet_error_sender.try_send(err) {
                    tracing::warn!("Failed to send error to faucet_error_sender: {:?}", e);
                }
                bail!("Claim failed, Unexpected VM status: {:?}", status)
            }
        }
    }

    async fn balance(&self) -> Result<U256> {
        let client = self.context.get_client().await?;
        let function_call =
            faucet_module::balance_call(self.faucet_module_address, self.faucet_object_id.clone());
        let response = client.rooch.execute_view_function(function_call).await?;
        match response.vm_status {
            VMStatusView::Executed => {
                let first_return = response
                    .return_values
                    .and_then(|mut values| values.pop())
                    .ok_or_else(|| anyhow::anyhow!("Get Balance failed, No return values"))?;
                let balance: U256 = bcs::from_bytes(&first_return.value.value.0)?;
                Ok(balance)
            }
            status => {
                bail!("Get Balance failed, Unexpected VM status: {:?}", status)
            }
        }
    }

    async fn check_claim(
        client: &Client,
        faucet_module_address: AccountAddress,
        faucet_object_id: ObjectID,
        claimer: AccountAddress,
        utxo_ids: Vec<ObjectID>,
    ) -> Result<U256> {
        tracing::debug!("check claim address: {}", claimer);

        let function_call = faucet_module::check_claim_function_call(
            faucet_module_address,
            faucet_object_id,
            claimer,
            utxo_ids,
        );
        execute_view_function(client, function_call).await
    }

    async fn get_utxos(client: &Client, address: UnitedAddressView) -> Result<Vec<ObjectID>> {
        let utxos = client
            .rooch
            .query_utxos(UTXOFilterView::Owner(address), None, None, Some(false))
            .await?;
        Ok(utxos
            .data
            .into_iter()
            .map(|utxo| utxo.metadata.id)
            .collect())
    }

    async fn fetch_tweet(&self, tweet_id: String) -> Result<ObjectID> {
        self.check_tweet(&tweet_id)?;
        let client = self.context.get_client().await?;
        let function_call = tweet_fetcher_module::fetch_tweet_function_call(
            self.faucet_module_address,
            tweet_id.clone(),
        );
        let tx_data = self
            .context
            .build_tx_data(
                self.faucet_sender,
                MoveAction::Function(function_call),
                None,
            )
            .await?;
        let tx = self.context.sign_transaction(self.faucet_sender, tx_data)?;
        let response = client.rooch.execute_tx(tx, None).await?;
        match response.execution_info.status {
            KeptVMStatusView::Executed => {
                let tweet_obj_id =
                    tweet_v2_module::tweet_object_id(self.faucet_module_address, tweet_id);
                Ok(tweet_obj_id)
            }
            status => bail!("Fetch tweet failed, Unexpected VM status: {:?}", status),
        }
    }

    async fn check_binding_tweet(&self, tweet_id: String) -> Result<BitcoinAddress> {
        self.check_tweet(tweet_id.as_str())?;
        let function_call = twitter_account_module::check_binding_tweet_function_call(
            self.faucet_module_address,
            tweet_id,
        );
        let client = self.context.get_client().await?;
        execute_view_function(&client, function_call).await
    }

    async fn verify_and_binding_twitter_account(&self, tweet_id: String) -> Result<BitcoinAddress> {
        self.check_tweet(tweet_id.as_str())?;
        let client = self.context.get_client().await?;
        let bitcoin_address = self.check_binding_tweet(tweet_id.clone()).await?;
        let function_call =
            twitter_account_module::verify_and_binding_twitter_account_function_call(
                self.faucet_module_address,
                tweet_id,
            );

        let tx_data = self
            .context
            .build_tx_data(
                self.faucet_sender,
                MoveAction::Function(function_call),
                None,
            )
            .await?;
        let tx = self.context.sign_transaction(self.faucet_sender, tx_data)?;
        let response = client.rooch.execute_tx(tx, None).await?;
        match response.execution_info.status {
            KeptVMStatusView::Executed => Ok(bitcoin_address),
            status => bail!(
                "Verify and binding twitter account failed, Unexpected VM status: {:?}",
                status
            ),
        }
    }

    async fn binding_twitter_account_with_inviter(
        &self,
        tweet_id: String,
        inviter: UnitedAddressView,
        claimer_sign: String,
        public_key: String,
        message: String,
    ) -> Result<BitcoinAddress> {
        let inviter_addr: AccountAddress = inviter.clone().into();
        self.check_tweet(tweet_id.as_str())?;
        let client = self.context.get_client().await?;
        let bitcoin_address = self.check_binding_tweet(tweet_id.clone()).await?;

        let function_call = invitation_module::claim_from_twitter_function_call(
            self.invitation_module_address,
            tweet_id,
            self.invitation_object_id.clone(),
            inviter_addr,
            public_key,
            claimer_sign,
            message,
        );

        let tx_data = self
            .context
            .build_tx_data(
                self.faucet_sender,
                MoveAction::Function(function_call),
                None,
            )
            .await?;
        let tx = self.context.sign_transaction(self.faucet_sender, tx_data)?;
        let response = client.rooch.execute_tx(tx, None).await?;
        match response.execution_info.status {
            KeptVMStatusView::Executed => Ok(bitcoin_address),
            status => bail!(
                "Verify and binding twitter account failed, Unexpected VM status: {:?}",
                status
            ),
        }
    }

    fn check_tweet(&self, tweet_id: &str) -> Result<()> {
        if tweet_id.len() != 19 {
            bail!("Invalid tweet id length: {}", tweet_id.len());
        }
        //TODO call twitter API to check tweet
        Ok(())
    }
}

async fn execute_view_function<T: DeserializeOwned>(
    client: &Client,
    function_call: FunctionCall,
) -> Result<T> {
    let response = client.rooch.execute_view_function(function_call).await?;
    match response.vm_status {
        VMStatusView::Executed => {
            let first_return = response
                .return_values
                .and_then(|mut values| values.pop())
                .ok_or_else(|| anyhow::anyhow!("No return values"))?;
            let result: T = bcs::from_bytes(&first_return.value.value.0)?;
            Ok(result)
        }
        VMStatusView::MoveAbort {
            location,
            abort_code,
        } => match location.0 {
            AbortLocation::Module(module_id) => {
                let reason = if faucet_module::MODULE_NAME == module_id.name() {
                    faucet_module::error_code_to_reason(abort_code.0)
                } else if twitter_account_module::MODULE_NAME == module_id.name() {
                    twitter_account_module::error_code_to_reason(abort_code.0)
                } else if tweet_fetcher_module::MODULE_NAME == module_id.name() {
                    tweet_fetcher_module::error_code_to_reason(abort_code.0)
                } else if tweet_v2_module::MODULE_NAME == module_id.name() {
                    tweet_v2_module::error_code_to_reason(abort_code.0)
                } else {
                    "Unknown".to_string()
                };
                bail!(
                    "Error in module: {}, code: {}, reason: {}",
                    module_id.name(),
                    abort_code.0,
                    reason
                )
            }
            _ => {
                bail!("Unknown abort location")
            }
        },
        status => {
            bail!("Unexpected VM status: {:?}", status)
        }
    }
}
