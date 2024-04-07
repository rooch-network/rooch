// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::messages::{ExecuteL1BlockMessage, ExecuteL2TxMessage};
use anyhow::Result;
use async_trait::async_trait;
use coerce::actor::{context::ActorContext, message::Handler, Actor};
use moveos_types::{
    moveos_std::{object::ObjectEntity, tx_context::TxContext},
    transaction::VerifiedMoveOSTransaction,
};
use rooch_executor::proxy::ExecutorProxy;
use rooch_indexer::proxy::IndexerProxy;
use rooch_proposer::proxy::ProposerProxy;
use rooch_sequencer::proxy::SequencerProxy;
use rooch_types::transaction::{
    ExecuteTransactionResponse, L1BlockWithBody, LedgerTransaction, LedgerTxData, RoochTransaction,
};

/// PipelineProcessor aggregates the executor, sequencer, proposer, and indexer to process transactions.
pub struct PipelineProcessorActor {
    pub(crate) executor: ExecutorProxy,
    pub(crate) sequencer: SequencerProxy,
    pub(crate) proposer: ProposerProxy,
    pub(crate) indexer: IndexerProxy,
    pub(crate) data_import_flag: bool,
}

impl PipelineProcessorActor {
    pub fn new(
        executor: ExecutorProxy,
        sequencer: SequencerProxy,
        proposer: ProposerProxy,
        indexer: IndexerProxy,
        data_import_flag: bool,
    ) -> Self {
        Self {
            executor,
            sequencer,
            proposer,
            indexer,
            data_import_flag,
        }
    }

    pub async fn execute_l1_block(
        &mut self,
        ctx: TxContext,
        l1_block: L1BlockWithBody,
    ) -> Result<ExecuteTransactionResponse> {
        let moveos_tx = self
            .executor
            .validate_l1_block(ctx, l1_block.clone())
            .await?;
        let ledger_tx = self
            .sequencer
            .sequence_transaction(LedgerTxData::L1Block(l1_block.block))
            .await?;
        self.execute_tx(ledger_tx, moveos_tx).await
    }

    pub async fn execute_l2_tx(
        &mut self,
        tx: RoochTransaction,
    ) -> Result<ExecuteTransactionResponse> {
        let moveos_tx = self.executor.validate_l2_tx(tx.clone()).await?;
        let ledger_tx = self
            .sequencer
            .sequence_transaction(LedgerTxData::L2Tx(tx))
            .await?;
        self.execute_tx(ledger_tx, moveos_tx).await
    }

    pub async fn execute_tx(
        &mut self,
        tx: LedgerTransaction,
        moveos_tx: VerifiedMoveOSTransaction,
    ) -> Result<ExecuteTransactionResponse> {
        // Then execute
        let (output, execution_info) = self.executor.execute_transaction(moveos_tx.clone()).await?;
        self.proposer
            .propose_transaction(tx.clone(), execution_info.clone())
            .await?;

        // Sync latest state root from writer executor to reader executor
        self.executor
            .refresh_state(
                ObjectEntity::root_object(execution_info.state_root, execution_info.size),
                output.is_upgrade,
            )
            .await?;

        let indexer = self.indexer.clone();
        let moveos_tx_clone = moveos_tx.clone();
        let execution_info_clone = execution_info.clone();
        let sequence_info = tx.sequence_info.clone();
        let output_clone = output.clone();

        // If bitcoin block data import, don't write all indexer
        // TODO put all indexer data into a single message
        if !self.data_import_flag {
            tokio::spawn(async move {
                let result = indexer
                    .indexer_states(tx.sequence_info.tx_order, output_clone.changeset.clone())
                    .await;
                match result {
                    Ok(_) => {}
                    Err(error) => log::error!("indexer states error: {}", error),
                };
                let result = indexer
                    .indexer_transaction(
                        tx.clone(),
                        execution_info_clone.clone(),
                        moveos_tx_clone.clone(),
                    )
                    .await;
                match result {
                    Ok(_) => {}
                    Err(error) => log::error!("indexer transactions error: {}", error),
                };
                let result = indexer
                    .indexer_events(output_clone.events.clone(), tx.clone(), moveos_tx_clone)
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
        self.execute_l1_block(msg.ctx, msg.tx).await
    }
}
