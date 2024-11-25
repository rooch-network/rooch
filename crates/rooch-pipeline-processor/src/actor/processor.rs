// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{
    ExecuteL1BlockMessage, ExecuteL1TxMessage, ExecuteL2TxMessage, GetServiceStatusMessage,
};
use crate::metrics::PipelineProcessorMetrics;
use anyhow::{Error, Result};
use async_trait::async_trait;
use bitcoin::hashes::Hash;
use coerce::actor::{context::ActorContext, message::Handler, Actor, LocalActorRef};
use function_name::named;
use moveos::moveos::VMPanicError;
use moveos_types::state::StateChangeSetExt;
use moveos_types::transaction::VerifiedMoveOSTransaction;
use prometheus::Registry;
use rooch_common::bitcoin_client::proxy::BitcoinClientProxy;
use rooch_da::actor::messages::AppendTransactionMessage;
use rooch_da::proxy::DAServerProxy;
use rooch_db::RoochDB;
use rooch_event::actor::{EventActor, UpdateServiceStatusMessage};
use rooch_executor::proxy::ExecutorProxy;
use rooch_indexer::proxy::IndexerProxy;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_types::bitcoin::types::Block as BitcoinBlock;
use rooch_types::{
    service_status::ServiceStatus,
    transaction::{
        ExecuteTransactionResponse, L1BlockWithBody, L1Transaction, LedgerTransaction,
        LedgerTxData, RoochTransaction,
    },
};
use std::sync::Arc;
use tracing::{debug, info};

/// PipelineProcessor aggregates the executor, sequencer, proposer, and indexer to process transactions.
pub struct PipelineProcessorActor {
    pub(crate) executor: ExecutorProxy,
    pub(crate) sequencer: SequencerProxy,
    pub(crate) da_server: DAServerProxy,
    pub(crate) indexer: IndexerProxy,
    pub(crate) service_status: ServiceStatus,
    pub(crate) metrics: Arc<PipelineProcessorMetrics>,
    event_actor: Option<LocalActorRef<EventActor>>,
    rooch_db: RoochDB,
    bitcoin_client_proxy: Option<BitcoinClientProxy>,
}

impl PipelineProcessorActor {
    pub fn new(
        executor: ExecutorProxy,
        sequencer: SequencerProxy,
        da_server: DAServerProxy,
        indexer: IndexerProxy,
        service_status: ServiceStatus,
        registry: &Registry,
        event_actor: Option<LocalActorRef<EventActor>>,
        rooch_db: RoochDB,
        bitcoin_client_proxy: Option<BitcoinClientProxy>,
    ) -> Self {
        Self {
            executor,
            sequencer,
            da_server,
            indexer,
            service_status,
            metrics: Arc::new(PipelineProcessorMetrics::new(registry)),
            event_actor,
            rooch_db,
            bitcoin_client_proxy,
        }
    }

    pub async fn process_sequenced_tx_on_startup(&mut self) -> Result<()> {
        let last_order = self.sequencer.get_sequencer_order().await.unwrap_or(0);
        debug!("process_sequenced_tx_on_startup last_order: {}", last_order);
        if last_order == 0 {
            return Ok(());
        }
        let mut txs = Vec::new();
        for order in (1..=last_order).rev() {
            let tx_hash = self
                .sequencer
                .get_tx_hashes(vec![order])
                .await?
                .pop()
                .flatten()
                .ok_or_else(|| anyhow::anyhow!("The tx with order {} should exists", order))?;
            let execution_info = self
                .executor
                .get_transaction_execution_infos_by_hash(vec![tx_hash])
                .await?
                .pop()
                .flatten();
            if execution_info.is_none() {
                txs.push(tx_hash);
            } else {
                //we scan the txs from the last to the first, so we can break when we find the first executed tx
                break;
            }
        }
        if txs.is_empty() {
            return Ok(());
        }
        info!(
            "Process sequenced but not executed transactions on startup, txs: {:?}",
            txs
        );

        for tx_hash in txs.into_iter().rev() {
            // reverse the txs to keep the order
            let ledger_tx = self
                .sequencer
                .get_transaction_by_hash(tx_hash)
                .await?
                .ok_or_else(|| anyhow::anyhow!("The tx with hash {} should exists", tx_hash))?;
            match &ledger_tx.data {
                LedgerTxData::L1Block(block) => {
                    debug!("process_sequenced_tx_on_startup l1_block_tx: {:?}", block);
                    match &self.bitcoin_client_proxy {
                        Some(bitcoin_client_proxy) => {
                            let block_hash_vec = block.block_hash.clone();
                            let block_hash =
                                bitcoin::block::BlockHash::from_slice(&block_hash_vec)?;
                            let btc_block = bitcoin_client_proxy.get_block(block_hash).await?;
                            let block_body = BitcoinBlock::from(btc_block);
                            let moveos_tx = self
                                .executor
                                .validate_l1_block(L1BlockWithBody::new(
                                    block.clone(),
                                    block_body.encode(),
                                ))
                                .await?;
                            self.execute_tx(ledger_tx.clone(), moveos_tx).await?;
                        }
                        None => {
                            return Err(anyhow::anyhow!(
                                "The bitcoin client proxy should be initialized before processing the sequenced l1_block_tx(block: {:?} on startup", block
                            ));
                        }
                    }
                }
                LedgerTxData::L1Tx(l1_tx) => {
                    debug!("process_sequenced_tx_on_startup l1_tx: {:?}", l1_tx);
                    let moveos_tx = self.executor.validate_l1_tx(l1_tx.clone()).await?;
                    self.execute_tx(ledger_tx.clone(), moveos_tx).await?;
                }
                LedgerTxData::L2Tx(l2_tx) => {
                    debug!("process_sequenced_tx_on_startup l2_tx: {:?}", l2_tx);
                    let moveos_tx = self.executor.validate_l2_tx(l2_tx.clone()).await?;
                    self.execute_tx(ledger_tx.clone(), moveos_tx).await?;
                }
            }
        }
        Ok(())
    }

    #[named]
    pub async fn execute_l1_block(
        &mut self,
        l1_block: L1BlockWithBody,
    ) -> Result<ExecuteTransactionResponse> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .pipeline_processor_execution_tx_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let moveos_tx = self.executor.validate_l1_block(l1_block.clone()).await?;
        let block_height = l1_block.block.block_height;
        let ledger_tx = self
            .sequencer
            .sequence_transaction(LedgerTxData::L1Block(l1_block.block))
            .await?;
        let tx_order = ledger_tx.sequence_info.tx_order;
        let size = moveos_tx.ctx.tx_size;
        let result = match self.execute_tx(ledger_tx, moveos_tx).await {
            Ok(v) => v,
            Err(err) => {
                if is_vm_panic_error(&err) {
                    tracing::error!(
                        "Execute L1 Block failed while VM panic occurred then \
                        set service to Maintenance mode and pause the relayer. error: {:?}, block {}, tx order {}",
                        err, block_height, tx_order
                    );
                    self.update_service_status(ServiceStatus::Maintenance).await;
                }
                return Err(err);
            }
        };

        let gas_used = result.output.gas_used;
        self.metrics
            .pipeline_processor_l1_block_gas_used
            .inc_by(gas_used);
        self.metrics
            .pipeline_processor_execution_tx_bytes
            .with_label_values(&[fn_name])
            .observe(size as f64);
        Ok(result)
    }

    #[named]
    pub async fn execute_l1_tx(
        &mut self,
        l1_tx: L1Transaction,
    ) -> Result<ExecuteTransactionResponse> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .pipeline_processor_execution_tx_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let moveos_tx = self.executor.validate_l1_tx(l1_tx.clone()).await?;
        let ledger_tx = self
            .sequencer
            .sequence_transaction(LedgerTxData::L1Tx(l1_tx.clone()))
            .await?;
        let size = moveos_tx.ctx.tx_size;
        let tx_order = ledger_tx.sequence_info.tx_order;
        let result = match self.execute_tx(ledger_tx, moveos_tx).await {
            Ok(v) => v,
            Err(err) => {
                if is_vm_panic_error(&err) {
                    tracing::error!(
                        "Execute L1 Tx failed while VM panic occurred then \
                        set service to Maintenance mode and pause the relayer. error: {:?}, tx order {}",
                        err, tx_order
                    );
                    self.update_service_status(ServiceStatus::Maintenance).await;
                }
                return Err(err);
            }
        };

        let gas_used = result.output.gas_used;
        self.metrics
            .pipeline_processor_l1_tx_gas_used
            .inc_by(gas_used);
        self.metrics
            .pipeline_processor_execution_tx_bytes
            .with_label_values(&[fn_name])
            .observe(size as f64);
        Ok(result)
    }

    async fn update_service_status(&mut self, status: ServiceStatus) {
        self.service_status = status;
        if let Some(event_actor) = self.event_actor.clone() {
            let _ = event_actor
                .send(UpdateServiceStatusMessage { status })
                .await;
        }
    }

    #[named]
    pub async fn execute_l2_tx(
        &mut self,
        mut tx: RoochTransaction,
    ) -> Result<ExecuteTransactionResponse> {
        debug!("pipeline execute_l2_tx: {:?}", tx.tx_hash());
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .pipeline_processor_execution_tx_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        let moveos_tx = self.executor.validate_l2_tx(tx.clone()).await?;
        let ledger_tx = self
            .sequencer
            .sequence_transaction(LedgerTxData::L2Tx(tx.clone()))
            .await?;
        let size = moveos_tx.ctx.tx_size;
        let result = match self.execute_tx(ledger_tx, moveos_tx).await {
            Ok(v) => v,
            Err(err) => {
                if is_vm_panic_error(&err) {
                    let l2_tx_bcs_bytes = bcs::to_bytes(&tx)?;
                    tracing::error!(
                        "Execute L2 Tx failed while VM panic occurred and revert tx. error: {:?} tx info {}",
                        err, hex::encode(l2_tx_bcs_bytes)
                    );
                    let tx_hash = tx.tx_hash();
                    self.rooch_db.revert_tx(tx_hash)?;
                }
                return Err(err);
            }
        };

        let gas_used = result.output.gas_used;
        self.metrics
            .pipeline_processor_l2_tx_gas_used
            .inc_by(gas_used);
        self.metrics
            .pipeline_processor_execution_tx_bytes
            .with_label_values(&[fn_name])
            .observe(size as f64);

        Ok(result)
    }

    #[named]
    pub async fn execute_tx(
        &mut self,
        tx: LedgerTransaction,
        mut moveos_tx: VerifiedMoveOSTransaction,
    ) -> Result<ExecuteTransactionResponse> {
        let fn_name = function_name!();
        let _timer = self
            .metrics
            .pipeline_processor_execution_tx_latency_seconds
            .with_label_values(&[fn_name])
            .start_timer();
        // Add sequence info to tx context, let the Move contract can get the sequence info
        moveos_tx.ctx.add(tx.sequence_info.clone())?;

        // Then execute
        let size = moveos_tx.ctx.tx_size;
        let (output, execution_info) = self.executor.execute_transaction(moveos_tx.clone()).await?;
        self.da_server
            .append_tx(AppendTransactionMessage {
                tx_order: tx.sequence_info.tx_order,
                tx_timestamp: tx.sequence_info.tx_timestamp,
            })
            .await?;
        let root = execution_info.root_metadata();
        // Sync latest state root from writer executor to reader executor
        self.executor
            .refresh_state(root.clone(), output.is_upgrade)
            .await?;
        // Save state change set is a notify call, do not block current task
        let state_change_set_ext =
            StateChangeSetExt::new(output.changeset.clone(), moveos_tx.ctx.sequence_number);
        self.executor
            .save_state_change_set(tx.sequence_info.tx_order, state_change_set_ext)
            .await?;

        let indexer = self.indexer.clone();
        let sequence_info = tx.sequence_info.clone();
        let execution_info_clone = execution_info.clone();
        let output_clone = output.clone();

        // If bitcoin block data import, don't write all indexer
        if !self.service_status.is_date_import_mode() {
            //The update_indexer is a notify call, do not block current task
            let result = indexer
                .update_indexer(
                    tx,
                    execution_info_clone,
                    moveos_tx,
                    output_clone.events,
                    output_clone.changeset,
                )
                .await;
            match result {
                Ok(_) => {}
                Err(error) => tracing::error!("Update indexer error: {}", error),
            };
        };

        self.metrics
            .pipeline_processor_execution_tx_bytes
            .with_label_values(&[fn_name])
            .observe(size as f64);

        Ok(ExecuteTransactionResponse {
            sequence_info,
            execution_info,
            output,
        })
    }
}

#[async_trait]
impl Actor for PipelineProcessorActor {}

#[async_trait]
impl Handler<ExecuteL2TxMessage> for PipelineProcessorActor {
    async fn handle(
        &mut self,
        msg: ExecuteL2TxMessage,
        _ctx: &mut ActorContext,
    ) -> Result<ExecuteTransactionResponse> {
        self.execute_l2_tx(msg.tx).await
    }
}

#[async_trait]
impl Handler<ExecuteL1BlockMessage> for PipelineProcessorActor {
    async fn handle(
        &mut self,
        msg: ExecuteL1BlockMessage,
        _ctx: &mut ActorContext,
    ) -> Result<ExecuteTransactionResponse> {
        self.execute_l1_block(msg.tx).await
    }
}

#[async_trait]
impl Handler<ExecuteL1TxMessage> for PipelineProcessorActor {
    async fn handle(
        &mut self,
        msg: ExecuteL1TxMessage,
        _ctx: &mut ActorContext,
    ) -> Result<ExecuteTransactionResponse> {
        self.execute_l1_tx(msg.tx).await
    }
}

#[async_trait]
impl Handler<GetServiceStatusMessage> for PipelineProcessorActor {
    async fn handle(
        &mut self,
        _msg: GetServiceStatusMessage,
        _ctx: &mut ActorContext,
    ) -> Result<ServiceStatus> {
        Ok(self.service_status)
    }
}

fn is_vm_panic_error(error: &Error) -> bool {
    if let Some(vm_error) = error.downcast_ref::<VMPanicError>() {
        match vm_error {
            VMPanicError::VerifierPanicError(_) | VMPanicError::SystemCallPanicError(_) => true,
        }
    } else {
        false
    }
}
