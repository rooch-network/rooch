// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::ExecuteTransactionMessage;
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use moveos_types::moveos_std::object::ObjectEntity;
use rooch_executor::proxy::ExecutorProxy;
use rooch_indexer::proxy::IndexerProxy;
use rooch_proposer::proxy::ProposerProxy;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_types::transaction::{ExecuteTransactionResponse, RoochTransaction};

/// PipelineProcessor aggregates the executor, sequencer, proposer, and indexer to process transactions.
pub struct PipelineProcessorActor {
    pub(crate) executor: ExecutorProxy,
    pub(crate) sequencer: SequencerProxy,
    pub(crate) proposer: ProposerProxy,
    pub(crate) indexer: IndexerProxy,
    pub(crate) data_verify_mode: bool,
}

impl PipelineProcessorActor {
    pub fn new(
        executor: ExecutorProxy,
        sequencer: SequencerProxy,
        proposer: ProposerProxy,
        indexer: IndexerProxy,
        data_verify_mode: bool,
    ) -> Self {
        Self {
            executor,
            sequencer,
            proposer,
            indexer,
            data_verify_mode,
        }
    }

    pub async fn execute_tx(&mut self, tx: RoochTransaction) -> Result<ExecuteTransactionResponse> {
        // First, validate the transaction
        let moveos_tx = self.executor.validate_transaction(tx.clone()).await?;
        let sequence_info = self.sequencer.sequence_transaction(tx.clone()).await?;
        // Then execute
        let (output, execution_info) = self.executor.execute_transaction(moveos_tx.clone()).await?;
        self.proposer
            .propose_transaction(tx.clone(), execution_info.clone(), sequence_info.clone())
            .await?;

        // Sync latest state root from writer executor to reader executor
        self.executor
            .refresh_state(
                ObjectEntity::root_object(execution_info.state_root, execution_info.size),
                output.is_upgrade,
            )
            .await?;

        let indexer = self.indexer.clone();
        let sequence_info_clone = sequence_info.clone();
        let moveos_tx_clone = moveos_tx.clone();
        let execution_info_clone = execution_info.clone();
        let output_clone = output.clone();

        // If data verify mode, don't write all indexer
        if !self.data_verify_mode {
            tokio::spawn(async move {
                let result = indexer
                    .indexer_states(sequence_info_clone.tx_order, output_clone.changeset.clone())
                    .await;
                match result {
                    Ok(_) => {}
                    Err(error) => log::error!("indexer states error: {}", error),
                };
                let result = indexer
                    .indexer_transaction(
                        tx.clone(),
                        sequence_info_clone.clone(),
                        execution_info_clone.clone(),
                        moveos_tx_clone.clone(),
                    )
                    .await;
                match result {
                    Ok(_) => {}
                    Err(error) => log::error!("indexer transactions error: {}", error),
                };
                let result = indexer
                    .indexer_events(
                        output_clone.events.clone(),
                        tx,
                        sequence_info_clone.clone(),
                        moveos_tx_clone,
                    )
                    .await;
                match result {
                    Ok(_) => {}
                    Err(error) => log::error!("indexer events error: {}", error),
                };
            });
        };

        Ok(ExecuteTransactionResponse {
            sequence_info,
            execution_info,
            output,
        })
    }
}

impl Actor for PipelineProcessorActor {}

#[async_trait]
impl Handler<ExecuteTransactionMessage> for PipelineProcessorActor {
    async fn handle(
        &mut self,
        msg: ExecuteTransactionMessage,
        _ctx: &mut ActorContext,
    ) -> Result<ExecuteTransactionResponse> {
        self.execute_tx(msg.tx).await
    }
}
