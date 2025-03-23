// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::errors::SubmitBatchError;
use crate::actor::messages::{
    AppendTransactionMessage, GetServerStatusMessage, RevertTransactionMessage,
};
use crate::backend::openda::AdapterSubmitStat;
use crate::backend::{DABackend, DABackends};
use crate::batcher::BatchMaker;
use anyhow::anyhow;
use async_trait::async_trait;
use coerce::actor::context::ActorContext;
use coerce::actor::message::Handler;
use coerce::actor::Actor;
use rooch_common::vec::validate_and_extract;
use rooch_config::da_config::{DAConfig, DEFAULT_DA_BACKGROUND_SUBMIT_INTERVAL};
use rooch_store::da_store::DAMetaStore;
use rooch_store::transaction_store::TransactionStore;
use rooch_store::RoochStore;
use rooch_types::crypto::RoochKeyPair;
use rooch_types::da::batch::{BlockRange, DABatch, SignedDABatchMeta};
use rooch_types::da::status::DAServerStatus;
use rooch_types::transaction::LedgerTransaction;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time;
use std::time::{Duration, SystemTime};
use tokio::sync::broadcast;

pub struct DAServerActor {
    rooch_store: RoochStore,
    backend_identifiers: Vec<String>,
    adapter_stats: Vec<AdapterSubmitStat>,
    last_block_number: Option<u128>,
    last_block_update_time: u64,
    background_last_block_update_time: Arc<AtomicU64>,
    batch_maker: BatchMaker,
}

impl Actor for DAServerActor {}

impl DAServerActor {
    pub async fn new(
        da_config: DAConfig,
        sequencer_key: RoochKeyPair,
        rooch_store: RoochStore,
        genesis_namespace: String,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> anyhow::Result<Self> {
        let min_block_to_submit = da_config.da_min_block_to_submit;
        let background_submit_interval = da_config
            .background_submit_interval
            .unwrap_or(DEFAULT_DA_BACKGROUND_SUBMIT_INTERVAL);

        let DABackends {
            backends,
            submit_threshold,
        } = DABackends::initialize(da_config.da_backend, genesis_namespace).await?;

        let backend_identifiers: Vec<String> = backends
            .iter()
            .map(|backend| backend.get_identifier())
            .collect();
        let adapter_stats = backends
            .iter()
            .map(|backend| backend.get_adapter_stats())
            .collect();

        let last_block_number = rooch_store.get_last_block_number()?;
        let background_last_block_update_time = Arc::new(AtomicU64::new(0));
        let server = DAServerActor {
            rooch_store: rooch_store.clone(),
            backend_identifiers,
            adapter_stats,
            last_block_number,
            last_block_update_time: 0,
            background_last_block_update_time: background_last_block_update_time.clone(),
            batch_maker: BatchMaker::new(rooch_store.clone()),
        };

        if submit_threshold != 0 {
            Self::run_background_submitter(
                rooch_store,
                sequencer_key,
                backends,
                submit_threshold,
                background_last_block_update_time,
                min_block_to_submit,
                background_submit_interval,
                shutdown_rx,
            );
        }

        Ok(server)
    }

    pub async fn get_status(&self) -> anyhow::Result<DAServerStatus> {
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

        let mut avail_backends = Vec::new();
        for (identifier, stat) in self
            .backend_identifiers
            .iter()
            .zip(self.adapter_stats.iter())
        {
            let identifier = identifier.clone();
            // Get the latest done chunk id
            // (it's block number too for ChunkV0, which is the only version now)
            let future = stat.get_latest_done_chunk_id();
            let result = future.await; // Resolve the future
            let result = result.unwrap_or(0); // for compatibility with old version which doesn't have this Optional field
            avail_backends.push((identifier, result));
        }

        Ok(DAServerStatus {
            last_block_number: self.last_block_number,
            last_tx_order,
            last_block_update_time,
            last_avail_block_number,
            last_avail_tx_order,
            last_avail_block_update_time,
            avail_backends,
        })
    }

    pub async fn append_transaction(
        &mut self,
        msg: AppendTransactionMessage,
    ) -> anyhow::Result<()> {
        let tx_order = msg.tx_order;
        let tx_timestamp = msg.tx_timestamp;
        let block_number_opt = self.batch_maker.append_transaction(tx_order, tx_timestamp);
        if let Some(block_number) = block_number_opt {
            self.last_block_number = Some(block_number);
            self.last_block_update_time = SystemTime::now()
                .duration_since(time::UNIX_EPOCH)?
                .as_secs();
        };
        Ok(())
    }

    pub async fn revert_transaction(
        &mut self,
        msg: RevertTransactionMessage,
    ) -> anyhow::Result<()> {
        let tx_order = msg.tx_order;
        self.batch_maker.revert_transaction(tx_order)?;
        Ok(())
    }

    // Spawns a background submitter to handle unsubmitted blocks off the main thread.
    // This prevents blocking other actor handlers and maintains the actor's responsiveness.
    fn run_background_submitter(
        rooch_store: RoochStore,
        sequencer_key: RoochKeyPair,
        backends: Vec<Arc<dyn DABackend>>,
        submit_threshold: usize,
        background_last_block_update_time: Arc<AtomicU64>,
        min_block_to_submit_opt: Option<u128>,
        background_submit_interval: u64,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) {
        tokio::spawn(async move {
            let background_submitter = BackgroundSubmitter {
                rooch_store: rooch_store.clone(),
                submitter: Submitter {
                    sequencer_key: sequencer_key.copy(),
                    rooch_store: rooch_store.clone(),
                    backends: backends.clone(),
                    submit_threshold,
                },
                last_block_update_time: background_last_block_update_time.clone(),
            };

            let mut old_last_block_number = None;
            let mut ticker = tokio::time::interval(Duration::from_secs(background_submit_interval));

            loop {
                tokio::select! {
                 _ = shutdown_rx.recv() => {
                     tracing::info!("DA Background Submitter thread received shutdown signal, exiting...");
                     break;
                 }
                 _ = ticker.tick() => {
                     match rooch_store.get_last_block_number() {
                         Ok(Some(last_block_number)) => {
                             if let Some(block_number_for_last_job) = old_last_block_number {
                                 if block_number_for_last_job > last_block_number {
                                          tracing::error!("da: last block number is smaller than last background job block number: {} < {}, database is inconsistent, will exit background submission",
                                              last_block_number, block_number_for_last_job);
                                          break;
                                      }
                                  }
                                  old_last_block_number = Some(last_block_number);

                                  if let Err(e) = background_submitter
                                      .start_job(min_block_to_submit_opt)
                                      .await
                                  {
                                      match e {
                                          SubmitBatchError::Recoverable(_) => {
                                              tracing::warn!("da: background submitter failed: {}", e);
                                          }
                                          SubmitBatchError::DatabaseInconsistent(_) => {
                                              tracing::error!("da: background submitter failed, will exit background submission: {}", e);
                                              break;
                                          }
                                      }
                                  }
                         }
                          Ok(None) => {
                                 if let Some(block_number_for_last_job) = old_last_block_number {
                                      tracing::error!("da: last block number is None, last background job block number: {}, database is inconsistent, will exit background submission",
                                          block_number_for_last_job);
                                      break;
                                  }
                              }
                              Err(e) => {
                                  tracing::error!("da: get last block number failed: {:?}", e);
                              }
                          }
                    }
                }
            }
        });
    }
}

#[async_trait]
impl Handler<GetServerStatusMessage> for DAServerActor {
    async fn handle(
        &mut self,
        _msg: GetServerStatusMessage,
        _ctx: &mut ActorContext,
    ) -> anyhow::Result<DAServerStatus> {
        self.get_status().await
    }
}

#[async_trait]
impl Handler<AppendTransactionMessage> for DAServerActor {
    async fn handle(
        &mut self,
        msg: AppendTransactionMessage,
        _ctx: &mut ActorContext,
    ) -> anyhow::Result<()> {
        self.append_transaction(msg).await
    }
}

#[async_trait]
impl Handler<RevertTransactionMessage> for DAServerActor {
    async fn handle(
        &mut self,
        msg: RevertTransactionMessage,
        _ctx: &mut ActorContext,
    ) -> anyhow::Result<()> {
        self.revert_transaction(msg).await
    }
}

pub(crate) struct Submitter {
    sequencer_key: RoochKeyPair,
    rooch_store: RoochStore,

    backends: Vec<Arc<dyn DABackend>>,
    submit_threshold: usize,
}

impl Submitter {
    async fn submit_batch_raw(
        &self,
        block_range: BlockRange,
        tx_list: Vec<LedgerTransaction>,
    ) -> anyhow::Result<SignedDABatchMeta, SubmitBatchError> {
        let block_number = block_range.block_number;
        let tx_order_start = block_range.tx_order_start;
        let tx_order_end = block_range.tx_order_end;

        // create batch
        let batch_ret = DABatch::new(
            block_number,
            tx_order_start,
            tx_order_end,
            &tx_list,
            &self.sequencer_key,
        );
        let batch = match batch_ret {
            Ok(batch) => batch,
            Err(e) => {
                return Err(SubmitBatchError::DatabaseInconsistent(e));
            }
        };
        let batch_meta = batch.meta.clone();
        let meta_signature = batch.meta_signature.clone();
        let batch_hash = batch.get_hash();

        // submit batch
        self.submit_batch_to_backends(batch)
            .await
            .map_err(SubmitBatchError::Recoverable)?;

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

        Ok(SignedDABatchMeta {
            meta: batch_meta,
            signature: meta_signature,
        })
    }

    async fn submit_batch_to_backends(&self, batch: DABatch) -> anyhow::Result<()> {
        let backends = self.backends.clone();
        let submit_threshold = self.submit_threshold;

        let batch = Arc::new(batch);

        // submit to backend in order until meet submit_threshold
        let mut success_count = 0;
        for backend in &backends {
            let submit_fut = backend.submit_batch(batch.clone());
            match submit_fut.await {
                Ok(_) => {
                    success_count += 1;
                    // TODO parallel submit
                    // if success_count >= submit_threshold {
                    //     break;
                    // }
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
    // start a job to submit unsubmitted blocks in this duration
    async fn start_job(
        &self,
        min_block_to_submit_opt: Option<u128>,
    ) -> anyhow::Result<(), SubmitBatchError> {
        // try to get unsubmitted blocks from [cursor, last_block_number]
        let origin_background_cursor = self
            .rooch_store
            .get_background_submit_block_cursor()
            .map_err(SubmitBatchError::Recoverable)?;
        let cursor_opt = Self::adjust_cursor(origin_background_cursor, min_block_to_submit_opt);

        let mut unsubmitted_blocks = self
            .rooch_store
            .get_submitting_blocks(cursor_opt.unwrap_or(0), None)
            .map_err(SubmitBatchError::Recoverable)?;

        // there is no unsubmitted block，wait for the next round
        if unsubmitted_blocks.is_empty() {
            return Ok(());
        }

        const DEFAULT_SUBMIT_INTERVAL: u64 = 2; // 2 seconds sleep between each block submission

        // ensure sorted by block number
        unsubmitted_blocks.sort_by(|a, b| a.block_number.cmp(&b.block_number));

        let mut done_count: u128 = 0;
        let mut max_block_number_submitted: u128 = 0;
        for unsubmitted_block_range in unsubmitted_blocks {
            let block_number = unsubmitted_block_range.block_number;
            let tx_order_start = unsubmitted_block_range.tx_order_start;
            let tx_order_end = unsubmitted_block_range.tx_order_end;
            // collect tx from start to end for rooch_store
            let tx_orders: Vec<u64> = (tx_order_start..=tx_order_end).collect();
            let tx_hashes_opt = self
                .rooch_store
                .get_tx_hashes(tx_orders.clone())
                .map_err(SubmitBatchError::Recoverable)?;
            let tx_hashes = validate_and_extract(tx_orders, tx_hashes_opt, |tx_order| {
                format!("Fail to get tx hash by tx_order: {}", tx_order)
            })
            .map_err(SubmitBatchError::DatabaseInconsistent)?;
            let tx_list_opt = self
                .rooch_store
                .get_transactions_by_hash(tx_hashes.clone())
                .map_err(SubmitBatchError::Recoverable)?;
            let tx_list = validate_and_extract(tx_hashes, tx_list_opt, |tx_hash| {
                format!("Fail to get tx by tx_hash: {:?}", tx_hash)
            })
            .map_err(SubmitBatchError::DatabaseInconsistent)?;

            self.submitter
                .submit_batch_raw(unsubmitted_block_range, tx_list)
                .await?;
            done_count += 1;
            max_block_number_submitted = block_number;
            tokio::time::sleep(Duration::from_secs(DEFAULT_SUBMIT_INTERVAL)).await;
        }

        if done_count == 0 {
            tracing::warn!("da: background submitting job failed: no blocks submitted after checking, should not happen");
        } else {
            self.update_cursor(max_block_number_submitted)
                .map_err(SubmitBatchError::Recoverable)?;
            tracing::info!(
            "da: background submitting job done: {} blocks submitted, new avail block number: {}",
            done_count,
            max_block_number_submitted);
        }

        Ok(())
    }

    // update to last_submitted_block_number, next submit state checking start from last_submitted_block_number
    fn update_cursor(&self, last_submitted_block_number: u128) -> anyhow::Result<()> {
        self.rooch_store
            .set_background_submit_block_cursor(last_submitted_block_number)?;
        self.last_block_update_time.store(
            SystemTime::now()
                .duration_since(time::UNIX_EPOCH)?
                .as_secs(),
            Ordering::Relaxed,
        );
        Ok(())
    }

    // adjust cursor(N) = max(cursor_opt, min_block_to_submit_opt), submit state checking start from N
    fn adjust_cursor(
        origin_cursor_opt: Option<u128>,
        min_block_to_submit_opt: Option<u128>,
    ) -> Option<u128> {
        if let Some(min_block_to_submit) = min_block_to_submit_opt {
            if let Some(origin_cursor) = origin_cursor_opt {
                if origin_cursor < min_block_to_submit {
                    return Some(min_block_to_submit);
                }
            } else {
                return Some(min_block_to_submit);
            }
        }
        origin_cursor_opt
    }
}

#[cfg(test)]
mod tests {
    use crate::actor::server::BackgroundSubmitter;

    #[test]
    fn test_background_submitter_adjust_cursor() {
        let cursor = Some(10);
        let min_block_to_submit = Some(5);
        let ret = BackgroundSubmitter::adjust_cursor(cursor, min_block_to_submit);
        assert_eq!(ret, Some(10));

        let cursor = Some(10);
        let min_block_to_submit = Some(15);
        let ret = BackgroundSubmitter::adjust_cursor(cursor, min_block_to_submit);
        assert_eq!(ret, Some(15));

        let cursor = Some(10);
        let min_block_to_submit = None;
        let ret = BackgroundSubmitter::adjust_cursor(cursor, min_block_to_submit);
        assert_eq!(ret, Some(10));

        let cursor = None;
        let min_block_to_submit = Some(5);
        let ret = BackgroundSubmitter::adjust_cursor(cursor, min_block_to_submit);
        assert_eq!(ret, Some(5));

        let cursor = None;
        let min_block_to_submit = None;
        let ret = BackgroundSubmitter::adjust_cursor(cursor, min_block_to_submit);
        assert_eq!(ret, None);
    }
}
