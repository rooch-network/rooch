// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::actor::messages::{AppendTransactionMessage, GetServerStatusMessage};
use crate::backend::openda::OpenDABackend;
use crate::backend::DABackend;
use crate::batcher::BatchMaker;
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
use rooch_types::da::state::DAServerStatus;
use rooch_types::transaction::LedgerTransaction;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time;
use std::time::{Duration, SystemTime};

// default background submit interval: 5 seconds
// smaller interval helps to reduce the delay of blocks making and submitting, get more accurate block number by status query
// the major duty of background submitter is to submit unsubmitted blocks made before server start,
// in most cases, backends work well enough to submit new blocks in time, which means after submitting old blocks,
// background submitter will have nothing to do.
// Only few database operations are needed to catch up with the latest block numbers,
// so it's okay to have a small interval.
const DEFAULT_BACKGROUND_SUBMIT_INTERVAL: u64 = 5;

pub struct DAServerActor {
    rooch_store: RoochStore,
    backend_names: Vec<String>,
    last_block_number: Option<u128>,
    last_block_update_time: u64,
    background_last_block_update_time: Arc<AtomicU64>,
    batch_maker: BatchMaker,
}

impl Actor for DAServerActor {}

struct ServerBackends {
    backends: Vec<Arc<dyn DABackend>>,
    backend_names: Vec<String>,
    submit_threshold: usize,
    is_nop_backend: bool,
    background_submit_interval: u64,
}

impl ServerBackends {
    const DEFAULT_SUBMIT_THRESHOLD: usize = 1;
    const DEFAULT_IS_NOP_BACKEND: bool = false;
    const DEFAULT_BACKGROUND_INTERVAL: u64 = DEFAULT_BACKGROUND_SUBMIT_INTERVAL;

    async fn process_backend_configs(
        backend_configs: &[DABackendConfigType],
        genesis_namespace: String,
        backends: &mut Vec<Arc<dyn DABackend>>,
        backend_names: &mut Vec<String>,
    ) -> anyhow::Result<usize> {
        let mut available_backends = 0;
        for backend_type in backend_configs {
            #[allow(irrefutable_let_patterns)]
            if let DABackendConfigType::OpenDa(openda_config) = backend_type {
                let backend = OpenDABackend::new(openda_config, genesis_namespace.clone()).await?;
                backends.push(Arc::new(backend));
                backend_names.push(format!("openda-{}", openda_config.scheme));
                available_backends += 1;
            }
        }
        Ok(available_backends)
    }

    async fn build(da_config: DAConfig, genesis_namespace: String) -> anyhow::Result<Self> {
        let mut backends: Vec<Arc<dyn DABackend>> = Vec::new();
        let mut backend_names: Vec<String> = Vec::new();
        let mut submit_threshold = Self::DEFAULT_SUBMIT_THRESHOLD;
        let mut is_nop_backend = Self::DEFAULT_IS_NOP_BACKEND;
        let background_submit_interval = da_config
            .da_backend
            .as_ref()
            .and_then(|backend_config| backend_config.background_submit_interval)
            .unwrap_or(Self::DEFAULT_BACKGROUND_INTERVAL);

        let mut available_backends_count = 1; // Nop is always available
        if let Some(mut backend_config) = da_config.da_backend {
            submit_threshold = backend_config.calculate_submit_threshold();
            available_backends_count = Self::process_backend_configs(
                &backend_config.backends,
                genesis_namespace,
                &mut backends,
                &mut backend_names,
            )
            .await?;
        } else {
            is_nop_backend = true;
            backends.push(Arc::new(crate::backend::DABackendNopProxy {}));
            backend_names.push("nop".to_string());
        }

        if available_backends_count < submit_threshold {
            return Err(anyhow!(
                "failed to start da: not enough backends for future submissions. exp>= {} act: {}",
                submit_threshold,
                available_backends_count
            ));
        }

        Ok(Self {
            backends,
            backend_names,
            submit_threshold,
            is_nop_backend,
            background_submit_interval,
        })
    }
}

impl DAServerActor {
    pub async fn new(
        da_config: DAConfig,
        sequencer_key: RoochKeyPair,
        rooch_store: RoochStore,
        genesis_namespace: String,
    ) -> anyhow::Result<Self> {
        let start_block_opt = da_config.da_start_block;
        let ServerBackends {
            backends,
            backend_names,
            submit_threshold,
            is_nop_backend,
            background_submit_interval,
        } = ServerBackends::build(da_config, genesis_namespace).await?;

        let last_block_number = rooch_store.get_last_block_number()?;
        let background_last_block_update_time = Arc::new(AtomicU64::new(0));
        let server = DAServerActor {
            rooch_store: rooch_store.clone(),
            backend_names,
            last_block_number,
            last_block_update_time: 0,
            background_last_block_update_time: background_last_block_update_time.clone(),
            batch_maker: BatchMaker::new(),
        };

        if !is_nop_backend {
            Self::create_background_submitter(
                rooch_store,
                sequencer_key,
                backends,
                submit_threshold,
                background_last_block_update_time,
                start_block_opt,
                background_submit_interval,
            );
        }

        Ok(server)
    }

    pub fn get_status(&self) -> anyhow::Result<DAServerStatus> {
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

        Ok(DAServerStatus {
            last_block_number: self.last_block_number,
            last_tx_order,
            last_block_update_time,
            last_avail_block_number,
            last_avail_tx_order,
            last_avail_block_update_time,
            avail_backends: self.backend_names.clone(),
        })
    }

    pub async fn append_transaction(
        &mut self,
        msg: AppendTransactionMessage,
    ) -> anyhow::Result<()> {
        let tx_order = msg.tx_order;
        let tx_timestamp = msg.tx_timestamp;
        let batch = self.batch_maker.append_transaction(tx_order, tx_timestamp);
        if let Some((tx_order_start, tx_order_end)) = batch {
            let block_number = self
                .rooch_store
                .append_submitting_block(tx_order_start, tx_order_end)?;
            self.last_block_number = Some(block_number);
            self.last_block_update_time = SystemTime::now()
                .duration_since(time::UNIX_EPOCH)?
                .as_secs();
        };
        Ok(())
    }

    fn create_background_submitter(
        rooch_store: RoochStore,
        sequencer_key: RoochKeyPair,
        backends: Vec<Arc<dyn DABackend>>,
        submit_threshold: usize,
        background_last_block_update_time: Arc<AtomicU64>,
        start_block_opt: Option<u128>,
        background_submit_interval: u64,
    ) {
        tokio::spawn(async move {
            let background_submitter = BackgroundSubmitter {
                rooch_store: rooch_store.clone(),
                submitter: Submitter {
                    sequencer_key: sequencer_key.copy(),
                    rooch_store: rooch_store.clone(),
                    nop_backend: false, // background submitter should not be nop-backend
                    backends: backends.clone(),
                    submit_threshold,
                },
                last_block_update_time: background_last_block_update_time.clone(),
            };

            let mut block_number_for_last_job = None;

            loop {
                match rooch_store.get_last_block_number() {
                    Ok(Some(last_block_number)) => {
                        if let Some(block_number_for_last_job) = block_number_for_last_job {
                            if block_number_for_last_job > last_block_number {
                                tracing::error!("da: last block number is smaller than last background job block number: {} < {}, database is inconsistent",
                                    last_block_number, block_number_for_last_job);
                                break;
                            }
                        }
                        block_number_for_last_job = Some(last_block_number);

                        if let Err(e) = background_submitter
                            .start_job(last_block_number, start_block_opt)
                            .await
                        {
                            tracing::error!("da: background submitter failed: {:?}", e);
                        }
                    }
                    Ok(None) => {
                        if let Some(block_number_for_last_job) = block_number_for_last_job {
                            tracing::error!("da: last block number is None, last background job block number: {}, database is inconsistent",
                                block_number_for_last_job);
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("da: get last block number failed: {:?}", e);
                    }
                }
                tokio::time::sleep(Duration::from_secs(background_submit_interval)).await;
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
        self.get_status()
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

pub(crate) struct Submitter {
    sequencer_key: RoochKeyPair,
    rooch_store: RoochStore,

    nop_backend: bool,
    backends: Vec<Arc<dyn DABackend>>,
    submit_threshold: usize,
}

impl Submitter {
    // TODO check all backends are idempotent or not, if not, we need to add a check to avoid duplicated submission
    // assume it's idempotent for now
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

    // adjust cursor(N) = max(cursor_opt, start_block), submit state checking start from N
    fn adjust_cursor(origin_cursor: Option<u128>, start_block_opt: Option<u128>) -> Option<u128> {
        if let Some(start_block) = start_block_opt {
            if let Some(cursor) = origin_cursor {
                if cursor < start_block {
                    return Some(start_block);
                }
            } else {
                return Some(start_block);
            }
        }
        origin_cursor
    }

    // start a job to submit unsubmitted blocks in this duration
    async fn start_job(
        &self,
        last_block_number: u128,
        start_block_opt: Option<u128>,
    ) -> anyhow::Result<()> {
        // try to get unsubmitted blocks from [cursor, last_block_number]
        let origin_background_cursor = self.rooch_store.get_background_submit_block_cursor()?;
        let cursor_opt = Self::adjust_cursor(origin_background_cursor, start_block_opt);
        let exp_count = if let Some(cursor) = cursor_opt {
            if cursor > last_block_number {
                return Err(anyhow!(
                    "background submitter cursor should not be larger than last_block_number: {} > {}, database is inconsistent",
                    cursor,
                    last_block_number
                ));
            }
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
    use crate::actor::server::{pair_tx_order_hash, BackgroundSubmitter};
    use moveos_types::h256::H256;

    #[test]
    fn test_background_submitter_adjust_cursor() {
        let cursor = Some(10);
        let start_block = Some(5);
        let ret = BackgroundSubmitter::adjust_cursor(cursor, start_block);
        assert_eq!(ret, Some(10));

        let cursor = Some(10);
        let start_block = Some(15);
        let ret = BackgroundSubmitter::adjust_cursor(cursor, start_block);
        assert_eq!(ret, Some(15));

        let cursor = Some(10);
        let start_block = None;
        let ret = BackgroundSubmitter::adjust_cursor(cursor, start_block);
        assert_eq!(ret, Some(10));

        let cursor = None;
        let start_block = Some(5);
        let ret = BackgroundSubmitter::adjust_cursor(cursor, start_block);
        assert_eq!(ret, Some(5));

        let cursor = None;
        let start_block = None;
        let ret = BackgroundSubmitter::adjust_cursor(cursor, start_block);
        assert_eq!(ret, None);
    }

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
