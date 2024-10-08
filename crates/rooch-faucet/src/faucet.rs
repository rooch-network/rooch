// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::faucet_module;
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
use moveos_types::transaction::MoveAction;
use prometheus::Registry;
use rooch_rpc_api::jsonrpc_types::btc::utxo::UTXOFilterView;
use rooch_rpc_api::jsonrpc_types::{KeptVMStatusView, UnitedAddressView, VMStatusView};
use rooch_rpc_client::wallet_context::WalletContext;
use rooch_rpc_client::Client;
use rooch_types::address::{ParsedAddress, RoochAddress};
use tokio::sync::mpsc::Sender;

#[derive(Parser, Debug, Clone)]
pub struct FaucetConfig {
    #[clap(long, default_value_t = 10000)]
    pub max_request_queue_length: u64,

    #[clap(long)]
    pub faucet_module_address: ParsedAddress,

    #[clap(long)]
    pub faucet_object_id: ObjectID,

    /// The address to send the faucet claim transaction
    /// Default is the active address in the wallet
    #[clap(long, default_value = "default")]
    pub faucet_sender: ParsedAddress,
}

pub struct Faucet {
    faucet_sender: RoochAddress,
    faucet_module_address: AccountAddress,
    faucet_object_id: ObjectID,
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
impl Handler<BalanceMessage> for Faucet {
    async fn handle(&mut self, _msg: BalanceMessage, _ctx: &mut ActorContext) -> Result<U256> {
        self.balance().await
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
        let faucet_sender = wallet_context.resolve_address(config.faucet_sender)?;
        Ok(Self {
            faucet_sender: faucet_sender.into(),
            faucet_module_address,
            faucet_object_id: config.faucet_object_id,
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
        let response = client.rooch.execute_view_function(function_call).await?;
        match response.vm_status {
            VMStatusView::Executed => {
                let first_return = response
                    .return_values
                    .and_then(|mut values| values.pop())
                    .ok_or_else(|| anyhow::anyhow!("Check claim failed, No return values"))?;
                let claim_amount: U256 = bcs::from_bytes(&first_return.value.value.0)?;
                Ok(claim_amount)
            }
            VMStatusView::MoveAbort {
                location,
                abort_code,
            } => match location.0 {
                AbortLocation::Module(module_id) => {
                    if module_id.name() == faucet_module::MODULE_NAME {
                        let reason = faucet_module::error_code_to_reason(abort_code.0);
                        bail!("Check claim failed, Module abort: {}", reason)
                    } else {
                        bail!(
                            "Check claim failed, Module abort in {}, abort_code: {}",
                            module_id,
                            abort_code
                        )
                    }
                }
                _ => {
                    bail!("Check claim failed, Unknown abort location")
                }
            },
            status => {
                bail!("Check claim failed, Unexpected VM status: {:?}", status)
            }
        }
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
}
