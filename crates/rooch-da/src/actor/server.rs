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
use coerce::actor::system::ActorSystem;
use coerce::actor::{Actor, IntoActor};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use moveos_types::h256;
use moveos_types::h256::H256;
use rooch_config::da_config::{DABackendConfigType, DAConfig};
use rooch_store::da_store::{DAMetaDBStore, DAMetaStore};
use rooch_store::transaction_store::TransactionStore;
use rooch_store::RoochStore;
use rooch_types::crypto::{RoochKeyPair, Signature};
use rooch_types::da::batch::{BlockRange, DABatch, DABatchMeta, SignedDABatchMeta};
use std::sync::{Arc, RwLock};

pub struct DAServerActor {
    sequencer_key: RoochKeyPair,
    rooch_store: RoochStore,
    backends: DABackends,
    last_block_number: Option<u128>,
}

struct DABackends {
    backends: Arc<RwLock<Vec<Arc<dyn DABackend + Send + Sync>>>>,
    submit_threshold: usize,
}

impl Actor for DAServerActor {}

impl DAServerActor {
    pub async fn new(
        da_config: DAConfig,
        sequencer_key: RoochKeyPair,
        rooch_store: RoochStore,
        last_tx_order: Option<u64>,
    ) -> anyhow::Result<Self> {
        let mut backends: Vec<Arc<dyn DABackend + Send + Sync>> = Vec::new();
        let mut submit_threshold = 1;
        let mut act_backends = 0;

        // backend config has higher priority than submit threshold
        if let Some(mut backend_config) = &da_config.da_backend {
            submit_threshold = backend_config.calculate_submit_threshold();

            for backend_type in &backend_config.backends {
                if let DABackendConfigType::Celestia(celestia_config) = backend_type {
                    let backend = CelestiaBackend::new(celestia_config).await?;
                    backends.push(Arc::new(backend));
                    act_backends += 1;
                }
                if let DABackendConfigType::OpenDa(openda_config) = backend_type {
                    let backend = OpenDABackend::new(openda_config).await?;
                    backends.push(Arc::new(backend));
                    act_backends += 1;
                }
            }
        } else {
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

        let sequencer_actor = DAServerActor {
            sequencer_key,
            rooch_store,
            backends: DABackends {
                backends: Arc::new(RwLock::new(backends)),
                submit_threshold,
            },
            last_block_number,
        };
        if let Some(last_block_number) = last_block_number {
            tokio::spawn(async move {
                if let Err(e) = sequencer_actor
                    .start_background_submit(last_block_number)
                    .await
                {
                    log::error!("{:?}, fail to start background da submit.", e);
                }
            });
        }

        Ok(sequencer_actor)
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
            .expect(
                format!(
                    "fail to append submitting block: last_block_number: {:?}; new block: tx_order_start: {:?}, tx_order_end: {:?}",
                    self.last_block_number, tx_order_start, tx_order_end
                )
                .as_str(),
            );

        let signed_meta = self.submit_batch_raw(
            BlockRange {
                block_number,
                tx_order_start,
                tx_order_end,
            },
            msg.tx_list_bytes,
        );
        self.last_block_number = Some(block_number);

        Ok(signed_meta)
    }

    async fn submit_batch_raw(
        &self,
        block_range: BlockRange,
        tx_list_bytes: Vec<u8>,
    ) -> anyhow::Result<SignedDABatchMeta> {
        let block_number = block_range.block_number;
        let tx_order_start = block_range.tx_order_start;
        let tx_order_end = block_range.tx_order_end;

        // create batch
        let tx_list_hash = h256::sha2_256_of(&tx_list_bytes);
        let batch_meta = DABatchMeta::new(block_number, tx_order_start, tx_order_end, tx_list_hash);
        let meta_bytes = bcs::to_bytes(&batch_meta)?;
        let meta_hash = h256::sha3_256_of(&meta_bytes);
        let meta_signature = Signature::sign(&meta_hash.0, &self.sequencer_key)
            .as_ref()
            .to_vec();
        let batch = DABatch {
            meta: batch_meta.clone(),
            meta_signature: meta_signature.clone(),
            tx_list_bytes,
        };

        // submit batch
        self.submit_batch_to_backends(batch).await?;

        // update block submitting state
        match self
            .rooch_store
            .set_submitting_block_done(block_number, tx_order_start, tx_order_end)
        {
            Ok(_) => {}
            Err(e) => {
                log::warn!("{:?}, fail to set submitting block done.", e);
            }
        };
        Ok(SignedDABatchMeta {
            meta: batch_meta,
            signature: meta_signature,
        })
    }

    async fn submit_batch_to_backends(&self, batch: DABatch) -> anyhow::Result<()> {
        let backends = self.backends.backends.read()?.to_vec();
        let submit_threshold = self.backends.submit_threshold;
        let mut submit_jobs = FuturesUnordered::new();
        for backend_arc in backends {
            let backend = Arc::clone(&backend_arc);
            submit_jobs.push(async move { backend.submit_batch(batch.clone()) });
        }
        let mut success_count = 0;
        while let Some(result) = submit_jobs.next().await {
            match result {
                Ok(_) => {
                    success_count += 1;
                    if success_count >= submit_threshold {
                        return Ok(());
                    }
                }
                Err(e) => {
                    log::warn!("{:?}, fail to submit batch to da server.", e);
                }
            }
        }
        if success_count < submit_threshold {
            return Err(anyhow::Error::msg(format!(
                "not enough successful submissions. exp>= {} act: {}",
                submit_threshold, success_count
            )));
        };
        Ok(())
    }

    // TODO: continue to submit blocks in the background even after all blocks <= init last_block_number have been submitted
    async fn start_background_submit(&self, stop_block_number: u128) -> anyhow::Result<()> {
        let cursor = self.rooch_store.get_background_submit_block_cursor()?;
        let unsubmitted_blocks = self
            .rooch_store
            .get_submitting_blocks(cursor.unwrap_or(0), None)?;

        if unsubmitted_blocks.is_empty() {
            return Ok(()); // nothing to do
        }

        let mut submit_count = 0;
        for block in unsubmitted_blocks {
            if block.block_number > stop_block_number {
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
            let mut tx_list_bytes = Vec::new();
            for tx_hash in tx_hashes {
                let tx = self
                    .rooch_store
                    .get_transaction_by_hash(tx_hash)?
                    .expect(format!("tx not found for hash: {:?}", tx_hash).as_str());
                tx_list_bytes.append(&mut tx.encode());
            }
            self.submit_batch_raw(block, tx_list_bytes).await?;
            submit_count += 1;
            if submit_count % 1024 == 0 {
                self.rooch_store
                    .set_background_submit_block_cursor(block.block_number)?;
                log::info!("da: submitted {} blocks in background", submit_count);
            }
        }
        self.rooch_store
            .set_background_submit_block_cursor(stop_block_number)?;
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
