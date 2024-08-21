// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{ExecuteL1BlockMessage, ExecuteL1TxMessage, ExecuteL2TxMessage};
use crate::metrics::PipelineProcessorMetrics;
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use function_name::named;
use moveos_types::transaction::VerifiedMoveOSTransaction;
use prometheus::Registry;
use rooch_executor::proxy::ExecutorProxy;
use rooch_indexer::proxy::IndexerProxy;
use rooch_proposer::proxy::ProposerProxy;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_types::transaction::TransactionSequenceInfoV1;
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
    pub(crate) proposer: ProposerProxy,
    pub(crate) indexer: IndexerProxy,
    pub(crate) service_status: ServiceStatus,
    pub(crate) metrics: Arc<PipelineProcessorMetrics>,
}

impl PipelineProcessorActor {
    pub fn new(
        executor: ExecutorProxy,
        sequencer: SequencerProxy,
        proposer: ProposerProxy,
        indexer: IndexerProxy,
        service_status: ServiceStatus,
        registry: &Registry,
    ) -> Self {
        Self {
            executor,
            sequencer,
            proposer,
            indexer,
            service_status,
            metrics: Arc::new(PipelineProcessorMetrics::new(registry)),
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

        for tx_hash in txs.into_iter() {
            let ledger_tx = self
                .sequencer
                .get_transaction_by_hash(tx_hash)
                .await?
                .ok_or_else(|| anyhow::anyhow!("The tx with hash {} should exists", tx_hash))?;
            match &ledger_tx.data {
                LedgerTxData::L1Block(_block) => {
                    //TODO how to get the L1BlockWithBody
                    unimplemented!("L1Block tx not support")
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
        let ledger_tx = self
            .sequencer
            .sequence_transaction(LedgerTxData::L1Block(l1_block.block))
            .await?;
        let size = moveos_tx.ctx.tx_size;
        let result = self.execute_tx(ledger_tx, moveos_tx).await?;

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
            .sequence_transaction(LedgerTxData::L1Tx(l1_tx))
            .await?;
        let size = moveos_tx.ctx.tx_size;
        let result = self.execute_tx(ledger_tx, moveos_tx).await?;

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
            .sequence_transaction(LedgerTxData::L2Tx(tx))
            .await?;
        let size = moveos_tx.ctx.tx_size;
        let result = self.execute_tx(ledger_tx, moveos_tx).await?;

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
        // We must add TransactionSequenceInfo and TransactionSequenceInfoV1 both to the tx_context because the rust code is upgraded first, then the framework is upgraded.
        // The old framework will read the TransactionSequenceInfoV1.
        let tx_sequence_info_v1 = TransactionSequenceInfoV1::from(tx.sequence_info.clone());
        moveos_tx.ctx.add(tx_sequence_info_v1)?;

        // Then execute
        let size = moveos_tx.ctx.tx_size;
        let (output, execution_info) = self.executor.execute_transaction(moveos_tx.clone()).await?;
        self.proposer
            .propose_transaction(tx.clone(), execution_info.clone())
            .await?;
        let root = execution_info.root_metadata();
        // Sync latest state root from writer executor to reader executor
        self.executor
            .refresh_state(root.clone(), output.is_upgrade)
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
                Err(error) => log::error!("Update indexer error: {}", error),
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
