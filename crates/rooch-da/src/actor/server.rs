// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{GetServerStatusMessage, PutDABatchMessage};
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
use rooch_types::da::state::ServerStatus;
use rooch_types::transaction::LedgerTransaction;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time;
use std::time::{Duration, SystemTime};

const DEFAULT_BACKGROUND_SUBMIT_INTERVAL: u64 = 5 * 60; // 5 minutes

pub struct DAServerActor {
    rooch_store: RoochStore,
    backend_names: Vec<String>,
    submitter: Submitter,
    last_block_number: Option<u128>,
    last_block_update_time: u64,
    background_last_block_update_time: Arc<AtomicU64>,
}

impl Actor for DAServerActor {}

impl DAServerActor {
    pub async fn new(
        da_config: DAConfig,
        sequencer_key: RoochKeyPair,
        rooch_store: RoochStore,
        genesis_namespace: String,
    ) -> anyhow::Result<Self> {
        let mut backends: Vec<Arc<dyn DABackend>> = Vec::new();
        let mut backend_names: Vec<String> = Vec::new();
        let mut submit_threshold = 1;
        let mut act_backends = 0;
        let mut background_submit_interval = DEFAULT_BACKGROUND_SUBMIT_INTERVAL;
        let mut nop_backend = false;
        // backend config has higher priority than submit threshold
        if let Some(mut backend_config) = da_config.da_backend {
            submit_threshold = backend_config.calculate_submit_threshold();

            for backend_type in &backend_config.backends {
                if let DABackendConfigType::Celestia(celestia_config) = backend_type {
                    let backend = CelestiaBackend::new(celestia_config).await?;
                    backends.push(Arc::new(backend));
                    backend_names.push("celestia".to_string());
                    act_backends += 1;
                }
                if let DABackendConfigType::OpenDa(openda_config) = backend_type {
                    let backend =
                        OpenDABackend::new(openda_config, genesis_namespace.clone()).await?;
                    backends.push(Arc::new(backend));
                    backend_names.push(format!("openda-{}", openda_config.scheme));
                    act_backends += 1;
                }
            }
            background_submit_interval =
                if let Some(interval) = backend_config.background_submit_interval {
                    interval
                } else {
                    DEFAULT_BACKGROUND_SUBMIT_INTERVAL
                };
        } else {
            nop_backend = true;
            backends.push(Arc::new(crate::backend::DABackendNopProxy {}));
            backend_names.push("nop".to_string());
            act_backends += 1;
        }

        if act_backends < submit_threshold {
            return Err(anyhow!(
                "failed to start da: not enough backends for future submissions. exp>= {} act: {}",
                submit_threshold,
                act_backends
            ));
        }

        let last_block_number = rooch_store.get_last_block_number()?;
        let background_last_block_update_time = Arc::new(AtomicU64::new(0));
        let server = DAServerActor {
            rooch_store: rooch_store.clone(),
            backend_names,
            submitter: Submitter {
                sequencer_key: sequencer_key.copy(),
                rooch_store: rooch_store.clone(),
                nop_backend,
                backends: backends.clone(),
                submit_threshold,
            },
            last_block_number,
            last_block_update_time: 0,
            background_last_block_update_time: background_last_block_update_time.clone(),
        };

        if !nop_backend {
            tokio::spawn(async move {
                let mut interval =
                    tokio::time::interval(Duration::from_secs(background_submit_interval)); // 10 minutes

                let background_submitter = BackgroundSubmitter {
                    rooch_store: rooch_store.clone(),
                    submitter: Submitter {
                        sequencer_key: sequencer_key.copy(),
                        rooch_store: rooch_store.clone(),
                        nop_backend,
                        backends: backends.clone(),
                        submit_threshold,
                    },
                    last_block_update_time: background_last_block_update_time.clone(),
                };

                loop {
                    interval.tick().await;
                    match rooch_store.get_last_block_number() {
                        Ok(Some(last_block_number)) => {
                            if let Err(e) = background_submitter.start_job(last_block_number).await
                            {
                                tracing::error!("da: background submitter failed: {:?}", e);
                            }
                        }
                        Ok(None) => {
                            tracing::info!("da: last block number is None");
                        }
                        Err(e) => {
                            tracing::error!("da: get last block number failed: {:?}", e);
                        }
                    }
                }
            });
        }

        Ok(server)
    }

    pub fn get_status(&self) -> anyhow::Result<ServerStatus> {
        let last_tx_order = if let Some(last_block_number) = self.last_block_number {
            let last_block_state = self.rooch_store.get_block_state(last_block_number)?;
            Some(last_block_state.block_range.tx_order_end)
        } else {
            None
        };
        let last_block_update_time = if self.last_block_update_time > 0 {
            Some(self.last_block_update_time)
        } else {
            None
        };

        let last_avail_block_number = self.rooch_store.get_background_submit_block_cursor()?;
        let last_avail_tx_order = if let Some(last_avail_block_number) = last_avail_block_number {
            let last_block_state = self.rooch_store.get_block_state(last_avail_block_number)?;
            Some(last_block_state.block_range.tx_order_end)
        } else {
            None
        };
        let background_last_block_update_time = self
            .background_last_block_update_time
            .load(Ordering::Relaxed);
        let last_avail_block_update_time = if background_last_block_update_time > 0 {
            Some(background_last_block_update_time)
        } else {
            None
        };

        Ok(ServerStatus {
            last_block_number: self.last_block_number,
            last_tx_order,
            last_block_update_time,
            last_avail_block_number,
            last_avail_tx_order,
            last_avail_block_update_time,
            avail_backends: self.backend_names.clone(),
        })
    }

    pub async fn submit_batch(
        &mut self,
        msg: PutDABatchMessage,
    ) -> anyhow::Result<SignedDABatchMeta> {
        let tx_order_start = msg.tx_order_start;
        let tx_order_end = msg.tx_order_end;
        let last_block_number = self.last_block_number;
        let (block_number, signed_meta) = self
            .submitter
            .submit_new_block(last_block_number, tx_order_start, tx_order_end, msg.tx_list)
            .await?;
        self.last_block_number = Some(block_number);
        self.last_block_update_time = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)?
            .as_secs();

        Ok(signed_meta)
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

#[async_trait]
impl Handler<GetServerStatusMessage> for DAServerActor {
    async fn handle(
        &mut self,
        _msg: GetServerStatusMessage,
        _ctx: &mut ActorContext,
    ) -> anyhow::Result<ServerStatus> {
        self.get_status()
    }
}

pub(crate) struct Submitter {
    sequencer_key: RoochKeyPair,
    rooch_store: RoochStore,

    nop_backend: bool,
    backends: Vec<Arc<dyn DABackend>>,
    submit_threshold: usize,
}

impl Submitter {
    async fn submit_new_block(
        &self,
        last_block_number: Option<u128>,
        tx_order_start: u64,
        tx_order_end: u64,
        tx_list: Vec<LedgerTransaction>,
    ) -> anyhow::Result<(u128, SignedDABatchMeta)> {
        let block_number = self.rooch_store.append_submitting_block(
            last_block_number,
            tx_order_start,
            tx_order_end,
        )?;

        let signed_meta = self
            .submit_batch_raw(
                BlockRange {
                    block_number,
                    tx_order_start,
                    tx_order_end,
                },
                tx_list,
            )
            .await?;
        Ok((block_number, signed_meta))
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
        let batch_hash = batch.get_hash();

        // submit batch
        self.submit_batch_to_backends(batch).await?;

        // update block submitting state if it's not nop-backend
        // if it's nop-backend, we don't need to update submitting state, we may need to submit batch to other backends later by fetch unsubmitted blocks
        if !self.nop_backend {
            match self.rooch_store.set_submitting_block_done(
                block_number,
                tx_order_start,
                tx_order_end,
                batch_hash,
            ) {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("{:?}, fail to set submitting block done.", e);
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
                    tracing::warn!("{:?}, fail to submit batch to backend.", e);
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
}

struct BackgroundSubmitter {
    rooch_store: RoochStore,
    submitter: Submitter,
    last_block_update_time: Arc<AtomicU64>,
}

impl BackgroundSubmitter {
    fn update_cursor(&self, cursor: u128) -> anyhow::Result<()> {
        self.rooch_store
            .set_background_submit_block_cursor(cursor)?;
        self.last_block_update_time.store(
            SystemTime::now()
                .duration_since(time::UNIX_EPOCH)?
                .as_secs(),
            Ordering::Relaxed,
        );
        Ok(())
    }

    async fn start_job(&self, last_block_number: u128) -> anyhow::Result<()> {
        // try to get unsubmitted blocks from [cursor, last_block_number]
        let cursor_opt = self.rooch_store.get_background_submit_block_cursor()?;
        let exp_count = if let Some(cursor) = cursor_opt {
            last_block_number - cursor // (cursor, last_block_number]
        } else {
            last_block_number + 1 // [0, last_block_number]
        };
        let unsubmitted_blocks = self
            .rooch_store
            .get_submitting_blocks(cursor_opt.unwrap_or(0), Some(exp_count as usize))?;

        // there is no unsubmitted block: [0, last_block_number]
        if unsubmitted_blocks.is_empty() {
            self.update_cursor(last_block_number)?;
            return Ok(());
        }
        // submitted: [0, first_unsubmitted_block - 1]
        let first_unsubmitted_block = unsubmitted_blocks.first().unwrap().block_number;
        if first_unsubmitted_block > 0 {
            let new_cursor = first_unsubmitted_block - 1;
            self.update_cursor(new_cursor)?;
        }

        let mut done_count: u128 = 0;
        let mut max_block_number_submitted: u128 = 0;
        for unsubmitted_block_range in unsubmitted_blocks {
            let block_number = unsubmitted_block_range.block_number;
            // leave last block to be submitted by submit_new_block, avoid duplicated submission
            if block_number >= last_block_number {
                break;
            }
            let tx_order_start = unsubmitted_block_range.tx_order_start;
            let tx_order_end = unsubmitted_block_range.tx_order_end;
            // collect tx from start to end for rooch_store
            let tx_orders: Vec<u64> = (tx_order_start..=tx_order_end).collect();
            let tx_hashes = self.rooch_store.get_tx_hashes(tx_orders.clone())?;
            let tx_order_hash_pairs = pair_tx_order_hash(tx_orders, tx_hashes)?;

            let mut tx_list: Vec<LedgerTransaction> = Vec::new();
            for (tx_order, tx_hash) in tx_order_hash_pairs {
                let tx = self
                    .rooch_store
                    .get_transaction_by_hash(tx_hash)?
                    .ok_or_else(|| {
                        anyhow!(
                            "fail to get transaction by tx_hash: {:?}, tx_order: {}",
                            tx_hash,
                            tx_order
                        )
                    })?; // should not happen
                tx_list.push(tx);
            }
            self.submitter
                .submit_batch_raw(unsubmitted_block_range, tx_list)
                .await?;
            done_count += 1;
            max_block_number_submitted = block_number;
            if done_count % 16 == 0 {
                // it's okay to set cursor a bit behind: submit_batch_raw set submitting block done, so it won't be submitted again after restart
                self.update_cursor(block_number)?;
            }
        }
        if done_count > 0 {
            self.update_cursor(max_block_number_submitted)?;
            tracing::info!(
                "da: background submitting job done: {} blocks submitted",
                done_count
            );
        } else {
            tracing::info!("da: background submitting job done: no blocks needed to submit");
        }

        Ok(())
    }
}

fn pair_tx_order_hash(
    tx_orders: Vec<u64>,
    tx_hashes: Vec<Option<H256>>,
) -> anyhow::Result<Vec<(u64, H256)>> {
    if tx_orders.len() != tx_hashes.len() {
        return Err(anyhow!("tx_orders and tx_hashes must have the same length"));
    }
    tx_orders
        .into_iter()
        .zip(tx_hashes)
        .map(|(tx_order, tx_hash)| {
            let tx_hash =
                tx_hash.ok_or_else(|| anyhow!("fail to get tx hash by tx_order: {}", tx_order))?;
            Ok((tx_order, tx_hash))
        })
        .collect::<anyhow::Result<Vec<(u64, H256)>>>()
}

#[cfg(test)]
mod tests {
    use crate::actor::server::pair_tx_order_hash;
    use moveos_types::h256::H256;

    #[test]
    fn pair_tx_order_hash_failed() {
        let tx_orders = vec![1, 2, 3];
        let tx_hashes = vec![Some(H256::from([1; 32])), None, Some(H256::from([3; 32]))];
        let ret = pair_tx_order_hash(tx_orders.clone(), tx_hashes);
        assert!(ret.is_err());

        let tx_hashes = vec![Some(H256::from([1; 32])), Some(H256::from([3; 32]))];
        let ret = pair_tx_order_hash(tx_orders.clone(), tx_hashes);
        assert!(ret.is_err());

        let tx_hashes = vec![
            Some(H256::from([1; 32])),
            Some(H256::from([2; 32])),
            Some(H256::from([3; 32])),
        ];
        let ret = pair_tx_order_hash(tx_orders.clone(), tx_hashes);
        assert!(ret.is_ok());
        let tx_order_hash_pairs = ret.unwrap();
        assert_eq!(tx_order_hash_pairs.len(), 3);
    }
}
