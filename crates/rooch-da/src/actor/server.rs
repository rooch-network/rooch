// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::PutDABatchMessage;
use crate::backend::celestia::CelestiaBackend;
use crate::backend::openda::OpenDABackend;
use crate::backend::DABackend;
use anyhow::anyhow;
use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::Handler;
use coerce::actor::Actor;
use moveos_types::h256::H256;
use rooch_config::da_config::{DABackendConfigType, DAConfig};
use rooch_store::da_store::DAMetaStore;
use rooch_store::transaction_store::TransactionStore;
use rooch_store::RoochStore;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::da::batch::{BlockRange, DABatch, SignedDABatchMeta};
use rooch_types::transaction::LedgerTransaction;
use std::sync::Arc;

pub struct DAServerActor {
    sequencer_key: RoochKeyPair,
    rooch_store: RoochStore,
    last_block_number: Option<u128>,

    nop_backend: bool,
    backends: Vec<Arc<dyn DABackend>>,
    submit_threshold: usize,
}

impl Actor for DAServerActor {}

impl DAServerActor {
    pub async fn new(
        da_config: DAConfig,
        sequencer_key: RoochKeyPair,
        rooch_store: RoochStore,
        last_tx_order: Option<u64>,
        genesis_namespace: String,
    ) -> anyhow::Result<Self> {
        let mut backends: Vec<Arc<dyn DABackend>> = Vec::new();
        let mut submit_threshold = 1;
        let mut act_backends = 0;

        let mut nop_backend = false;
        // backend config has higher priority than submit threshold
        if let Some(mut backend_config) = da_config.da_backend {
            submit_threshold = backend_config.calculate_submit_threshold();

            for backend_type in &backend_config.backends {
                if let DABackendConfigType::Celestia(celestia_config) = backend_type {
                    let backend = CelestiaBackend::new(celestia_config).await?;
                    backends.push(Arc::new(backend));
                    act_backends += 1;
                }
                if let DABackendConfigType::OpenDa(openda_config) = backend_type {
                    let backend =
                        OpenDABackend::new(openda_config, genesis_namespace.clone()).await?;
                    backends.push(Arc::new(backend));
                    act_backends += 1;
                }
            }
        } else {
            nop_backend = true;
            backends.push(Arc::new(crate::backend::DABackendNopProxy {}));
            act_backends += 1;
        }

        if act_backends < submit_threshold {
            return Err(anyhow!(
                "failed to start da: not enough backends for future submissions. exp>= {} act: {}",
                submit_threshold,
                act_backends
            ));
        }

        rooch_store.catchup_submitting_blocks(last_tx_order)?;
        let last_block_number = rooch_store.get_last_block_number()?;

        if let Some(last_block_number) = last_block_number {
            if !nop_backend {
                let background_da_server = DAServerActor {
                    sequencer_key: sequencer_key.copy(),
                    rooch_store: rooch_store.clone(),
                    last_block_number: Some(last_block_number),
                    nop_backend,
                    backends: backends.clone(),
                    submit_threshold,
                };

                tokio::spawn(async move {
                    if let Err(e) = background_da_server
                        .start_background_submit(last_block_number)
                        .await
                    {
                        log::error!("{:?}, fail to start background da submit.", e);
                    }
                });
            }
        }

        Ok(DAServerActor {
            sequencer_key,
            rooch_store,
            last_block_number,
            nop_backend,
            backends,
            submit_threshold,
        })
    }

    pub async fn submit_batch(
        &mut self,
        msg: PutDABatchMessage,
    ) -> anyhow::Result<SignedDABatchMeta> {
        let tx_order_start = msg.tx_order_start;
        let tx_order_end = msg.tx_order_end;
        let block_number = self
            .rooch_store
            .append_submitting_block(self.last_block_number, tx_order_start, tx_order_end)
            .unwrap_or_else(|_| panic!("fail to append submitting block: last_block_number: {:?}; new block: tx_order_start: {:?}, tx_order_end: {:?}",
                   self.last_block_number, tx_order_start, tx_order_end));

        let signed_meta = self
            .submit_batch_raw(
                BlockRange {
                    block_number,
                    tx_order_start,
                    tx_order_end,
                },
                msg.tx_list,
            )
            .await?;
        self.last_block_number = Some(block_number);

        Ok(signed_meta)
    }

    // should be idempotent
    async fn submit_batch_raw(
        &self,
        block_range: BlockRange,
        tx_list: Vec<LedgerTransaction>,
    ) -> anyhow::Result<SignedDABatchMeta> {
        let block_number = block_range.block_number;
        let tx_order_start = block_range.tx_order_start;
        let tx_order_end = block_range.tx_order_end;

        // create batch
        let batch = DABatch::new(
            block_number,
            tx_order_start,
            tx_order_end,
            &tx_list,
            self.sequencer_key.copy(),
        );
        let batch_meta = batch.meta.clone();
        let meta_signature = batch.meta_signature.clone();

        // submit batch
        self.submit_batch_to_backends(batch).await?;

        // update block submitting state if it's not nop-backend
        // if it's nop-backend, we don't need to update submitting state, we may need to submit batch to other backends later by fetch unsubmitted blocks
        if !self.nop_backend {
            match self.rooch_store.set_submitting_block_done(
                block_number,
                tx_order_start,
                tx_order_end,
            ) {
                Ok(_) => {}
                Err(e) => {
                    log::warn!("{:?}, fail to set submitting block done.", e);
                }
            };
        };
        Ok(SignedDABatchMeta {
            meta: batch_meta,
            signature: meta_signature,
        })
    }

    async fn submit_batch_to_backends(&self, batch: DABatch) -> anyhow::Result<()> {
        let backends = self.backends.clone();
        let submit_threshold = self.submit_threshold;
        // submit to backend in order until meet submit_threshold
        let mut success_count = 0;
        for backend in &backends {
            let submit_fut = backend.submit_batch(batch.clone());
            match submit_fut.await {
                Ok(_) => {
                    success_count += 1;
                    if success_count >= submit_threshold {
                        break;
                    }
                }
                Err(e) => {
                    log::warn!("{:?}, fail to submit batch to backend.", e);
                }
            }
        }

        if success_count < submit_threshold {
            return Err(anyhow!(
                "not enough successful submissions. exp>= {} act: {}",
                submit_threshold,
                success_count
            ));
        };
        Ok(())
    }

    // TODO: continue to submit blocks in the background even after all blocks <= init last_block_number have been submitted
    async fn start_background_submit(&self, last_block_number: u128) -> anyhow::Result<()> {
        let cursor = self.rooch_store.get_background_submit_block_cursor()?;
        let exp_count = if let Some(cursor) = cursor {
            last_block_number - cursor
        } else {
            last_block_number + 1
        };
        let unsubmitted_blocks = self
            .rooch_store
            .get_submitting_blocks(cursor.unwrap_or(0), Some(exp_count as usize))?;

        if unsubmitted_blocks.is_empty() {
            return Ok(()); // nothing to do
        }

        let mut submit_count: u128 = 0;
        for block in unsubmitted_blocks {
            let block_number = block.block_number;
            if block_number > last_block_number {
                break;
            }
            let tx_order_start = block.tx_order_start;
            let tx_order_end = block.tx_order_end;
            // collect tx from start to end for rooch_store
            let tx_orders: Vec<u64> = (tx_order_start..=tx_order_end).collect();
            let tx_hashes = self.rooch_store.get_tx_hashes(tx_orders)?;
            let tx_hashes: Vec<H256> = tx_hashes
                .into_iter()
                .map(|tx_hash| tx_hash.unwrap())
                .collect();
            let mut tx_list: Vec<LedgerTransaction> = Vec::new();
            for tx_hash in tx_hashes {
                let tx = self
                    .rooch_store
                    .get_transaction_by_hash(tx_hash)?
                    .unwrap_or_else(|| panic!("tx not found for hash: {:?}", tx_hash));
                tx_list.push(tx);
            }
            self.submit_batch_raw(block, tx_list).await?;
            submit_count += 1;
            if submit_count % 1024 == 0 {
                // it's okay to set cursor a bit behind: submit_batch_raw set submitting block done, so it won't be submitted again after restart
                self.rooch_store
                    .set_background_submit_block_cursor(block_number)?;
                log::info!("da: submitted {} blocks in background", submit_count);
            }
        }
        self.rooch_store
            .set_background_submit_block_cursor(last_block_number)?;
        Ok(())
    }
}

#[async_trait]
impl Handler<PutDABatchMessage> for DAServerActor {
    async fn handle(
        &mut self,
        msg: PutDABatchMessage,
        _ctx: &mut ActorContext,
    ) -> anyhow::Result<SignedDABatchMeta> {
        self.submit_batch(msg).await
    }
}
