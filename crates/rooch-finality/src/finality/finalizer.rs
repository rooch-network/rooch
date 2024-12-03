// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::rpc_client::FinalityGadgetGrpcClient;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rooch_db::RoochDB;
use rooch_types::finality_block::{Block, BlockID, L1BlockRef, L2BlockRef};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, error};

// defaultFinalityLookback defines the amount of L1<>L2 relations to track for finalization purposes, one per L1 block.
//
// When L1 finalizes blocks, it finalizes finalityLookback blocks behind the L1 head.
// Non-finality may take longer, but when it does finalize again, it is within this range of the L1 head.
// Thus we only need to retain the L1<>L2 derivation relation data of this many L1 blocks.
//
// In the event of older finalization signals, misconfiguration, or insufficient L1<>L2 derivation relation data,
// then we may miss the opportunity to finalize more L2 blocks.
// This does not cause any divergence, it just causes lagging finalization status.
//
// The beacon chain on mainnet has 32 slots per epoch,
// and new finalization events happen at most 4 epochs behind the head.
// And then we add 1 to make pruning easier by leaving room for a new item without pruning the 32*4.
const DEFAULT_FINALITY_LOOKBACK: u64 = 4 * 32 + 1;

// finalityDelay is the number of L1 blocks to traverse before trying to finalize L2 blocks again.
// We do not want to do this too often, since it requires fetching a L1 block by number, so no cache data.
const FINALITY_DELAY: u64 = 64;

// // Types and Structs
// #[derive(Clone, Debug, Default, PartialEq)]
// pub struct BlockID {
//     pub number: u64,
//     pub hash: String,
// }
//
// #[derive(Clone, Debug, Default, PartialEq)]
// pub struct L1BlockRef {
//     pub hash: String,
//     pub number: u64,
//     pub time: u64,
//     pub parent_hash: String,
// }
//
// #[derive(Clone, Debug, Default, PartialEq)]
// pub struct L2BlockRef {
//     pub hash: H256,
//     pub number: u64,
//     pub time: u64,
//     pub parent_hash: H256,
// }

#[derive(Clone, Debug)]
pub struct FinalityData {
    // The last L2 block that was fully derived and inserted into the L2 engine while processing this L1 block.
    l2_block: L2BlockRef,
    // The L1 block this stage was at when inserting the L2 block.
    // When this L1 block is finalized, the L2 chain up to this block can be fully reproduced from finalized L1 data.
    l1_block: BlockID,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub babylon_finality_gadget_rpc: String,
}

// Events
#[derive(Debug)]
pub enum Event {
    FinalizeL1(FinalizeL1Event),
    SafeDerived(SafeDerivedEvent),
    DeriverIdle(DeriverIdleEvent),
    Reset(ResetEvent),
    TryFinalize,
    ForkchoiceUpdate(ForkchoiceUpdateEvent),
    CriticalError(CriticalErrorEvent),
}

#[derive(Debug)]
pub struct FinalizeL1Event {
    pub finalized_l1: L1BlockRef,
}

#[derive(Debug)]
pub struct SafeDerivedEvent {
    pub safe: L2BlockRef,
    pub derived_from: L1BlockRef,
}

#[derive(Debug)]
pub struct DeriverIdleEvent {
    pub origin: L1BlockRef,
}

#[derive(Debug)]
pub struct ResetEvent {
    pub err: String,
}

#[derive(Debug)]
pub struct ForkchoiceUpdateEvent {
    pub finalized_l2_head: L2BlockRef,
}

#[derive(Debug)]
pub struct CriticalErrorEvent {
    pub err: String,
}

// Traits
#[async_trait]
pub trait FinalizerEngine: Send + Sync {
    fn finalized(&self) -> L2BlockRef;
    fn set_finalized_head(&self, head: L2BlockRef);
}

#[async_trait]
pub trait FinalizerL1Interface: Send + Sync {
    // async fn l1_block_ref_by_number(&self, number: u64) -> Result<L1BlockRef, Box<dyn std::error::Error + Send + Sync>>;
    async fn l1_block_ref_by_number(&self, number: u64) -> Result<L1BlockRef>;
}
#[async_trait]
pub trait EventEmitter: Send + Sync {
    async fn emit(&self, event: Event);
}

// Main Finalizer Implementation
pub struct Finalizer {
    ctx: Arc<tokio::sync::Mutex<()>>,
    emitter: Arc<dyn EventEmitter>,
    finalized_l1: Arc<Mutex<L1BlockRef>>,
    last_finalized_l2: Arc<Mutex<L2BlockRef>>,
    tried_finalize_at: Arc<Mutex<u64>>,
    finality_data: Arc<Mutex<Vec<FinalityData>>>,
    finality_lookback: u64,
    l1_fetcher: Arc<dyn FinalizerL1Interface>,
    l2_fetcher: RoochDB,
    babylon_finality_client: FinalityGadgetGrpcClient,
}

impl Finalizer {
    fn calc_finality_lookback(_cfg: &Config) -> u64 {
        DEFAULT_FINALITY_LOOKBACK
    }

    pub async fn new(
        cfg: &Config,
        l1_fetcher: Arc<dyn FinalizerL1Interface>,
        l2_fetcher: RoochDB,
    ) -> Result<Self, anyhow::Error> {
        let lookback = Self::calc_finality_lookback(cfg);

        debug!(
            "creating Babylon Finality client, rpc_addr {:?}",
            cfg.babylon_finality_gadget_rpc
        );

        let babylon_finality_gadget_client =
            FinalityGadgetGrpcClient::new(cfg.babylon_finality_gadget_rpc.clone())
                .await
                .map_err(|e| anyhow!(format!("New finalizer error: {:?}", e)))?;

        Ok(Finalizer {
            ctx: Arc::new(tokio::sync::Mutex::new(())),
            emitter: Arc::new(NoopEmitter::default()),
            finalized_l1: Arc::new(Mutex::new(L1BlockRef::default())),
            last_finalized_l2: Arc::new(Mutex::new(L2BlockRef::default())),
            tried_finalize_at: Arc::new(Mutex::new(0)),
            finality_data: Arc::new(Mutex::new(Vec::with_capacity(lookback as usize))),
            finality_lookback: lookback,
            l1_fetcher,
            l2_fetcher,
            babylon_finality_client: babylon_finality_gadget_client,
        })
    }

    pub fn attach_emitter(&mut self, emitter: Arc<dyn EventEmitter>) {
        self.emitter = emitter;
    }

    pub fn finalized_l1(&self) -> L1BlockRef {
        self.finalized_l1.lock().unwrap().clone()
    }

    pub async fn on_event(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::FinalizeL1(ev) => {
                self.on_l1_finalized(ev.finalized_l1).await;
                Ok(true)
            }
            Event::SafeDerived(ev) => {
                self.on_derived_safe_block(ev.safe, ev.derived_from).await;
                Ok(true)
            }
            Event::DeriverIdle(ev) => {
                self.on_derivation_idle(ev.origin).await;
                Ok(true)
            }
            Event::Reset(_) => {
                self.on_reset().await;
                Ok(true)
            }
            Event::TryFinalize => {
                self.try_finalize().await?;
                Ok(true)
            }
            Event::ForkchoiceUpdate(ev) => {
                *self.last_finalized_l2.lock().unwrap() = ev.finalized_l2_head;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    // onL1Finalized applies a L1 finality signal
    async fn on_l1_finalized(&self, l1_origin: L1BlockRef) {
        let mut finalized_l1 = self.finalized_l1.lock().unwrap();
        let prev_finalized_l1 = finalized_l1.clone();

        if l1_origin.number < finalized_l1.number {
            error!("ignoring old L1 finalized block signal! prev_finalized_l1 {:?}, signaled_finalized_l1 {:?}", prev_finalized_l1, l1_origin);
            return;
        }

        if *finalized_l1 != l1_origin {
            *self.tried_finalize_at.lock().unwrap() = 0;
            *finalized_l1 = l1_origin;
        }

        self.emitter.emit(Event::TryFinalize).await;
    }

    // onDerivationIdle is called when the pipeline is exhausted of new data (i.e. no more L2 blocks to derive from).
    async fn on_derivation_idle(&self, derived_from: L1BlockRef) {
        let finalized_l1 = self.finalized_l1.lock().unwrap();
        if *finalized_l1 == L1BlockRef::default() {
            return;
        }

        let tried_finalize_at = *self.tried_finalize_at.lock().unwrap();
        if tried_finalize_at != 0 && derived_from.number <= tried_finalize_at + FINALITY_DELAY {
            return;
        }

        debug!("processing L1 finality information, l1_finalized {:?}, derived_from {:?}, previous {:?}", finalized_l1, derived_from, tried_finalize_at);

        *self.tried_finalize_at.lock().unwrap() = derived_from.number;
        self.emitter.emit(Event::TryFinalize).await;
    }

    pub async fn try_finalize(&mut self) -> Result<()> {
        // Clone or copy values that need to be used across await points
        let finalized_l1 = {
            let guard = self.finalized_l1.lock().unwrap();
            guard.clone()
        };
        let gadget_activated_timestamp = match self
            .babylon_finality_client
            .query_btc_staking_activated_timestamp()
            .await
        {
            Ok(timestamp) => timestamp,
            Err(e) if e.to_string().contains("BtcStakingNotActivated") => 0,
            Err(e) => {
                self.emitter
                    .emit(Event::CriticalError(CriticalErrorEvent {
                        err: format!(
                            "failed to query BTC staking activated timestamp: {}",
                            e.to_string()
                        ),
                    }))
                    .await;
                return Ok(());
            }
        };

        // let mut finalized_l2 = self.last_finalized_l2.lock().unwrap().clone();
        let mut finalized_l2 = {
            let guard = self.last_finalized_l2.lock().unwrap();
            guard.clone()
        };
        let mut finalized_derived_from = None;

        let finality_data = self.finality_data.lock().unwrap().clone();
        for fd in finality_data.iter() {
            if fd.l2_block.number > finalized_l2.number && fd.l1_block.number <= finalized_l1.number
            {
                if let Some(last_finalized_block) = self
                    .find_last_btc_finalized_l2_block(
                        fd.l2_block.number,
                        finalized_l2.number,
                        gadget_activated_timestamp,
                    )
                    .await?
                {
                    finalized_l2 = last_finalized_block;
                    finalized_derived_from = Some(fd.l1_block.clone());
                }

                if finalized_derived_from.is_none() || finalized_l2.number != fd.l2_block.number {
                    break;
                }
            }
        }

        if let Some(derived_from) = finalized_derived_from {
            let ctx = tokio::time::timeout(Duration::from_secs(10), async {
                let signal_ref = self
                    .l1_fetcher
                    .l1_block_ref_by_number(finalized_l1.number)
                    .await
                    .map_err(|e| {
                        anyhow!(format!("l1_block_ref_by_number error: {:?}", e.to_string()))
                    })?;

                if signal_ref.hash != finalized_l1.hash {
                    let err_msg = format!(
                        "need to reset, we assumed {:?} is finalized, but canonical chain is {:?}",
                        finalized_l1, signal_ref
                    );
                    self.emitter
                        .emit(Event::Reset(ResetEvent { err: err_msg }))
                        .await;
                    return Err(anyhow::anyhow!("Chain reset needed"));
                }

                let derived_ref = self
                    .l1_fetcher
                    .l1_block_ref_by_number(derived_from.number)
                    .await?;

                if derived_ref.hash != derived_from.hash {
                    let err_msg = format!(
                        "need to reset, we are on {:?}, not on the finalizing L1 chain {:?}",
                        derived_from, derived_ref
                    );
                    self.emitter
                        .emit(Event::Reset(ResetEvent { err: err_msg }))
                        .await;
                    return Err(anyhow::anyhow!("Chain reset needed"));
                }
                Ok(())
                // Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            })
            .await;

            match ctx {
                Ok(Ok(())) => {
                    self.emitter
                        .emit(Event::ForkchoiceUpdate(ForkchoiceUpdateEvent {
                            finalized_l2_head: finalized_l2,
                        }))
                        .await;
                }
                Ok(Err(e)) => {
                    return Err(anyhow::anyhow!(
                        "Error during finalization, error {:?}",
                        e.to_string()
                    ));
                }
                Err(_) => {
                    return Err(anyhow::anyhow!("Timeout during finalization"));
                }
            }
        }
        return Ok(());
    }

    async fn find_last_btc_finalized_l2_block(
        &mut self,
        fd_l2_block_number: u64,
        finalized_l2_number: u64,
        gadget_activated_timestamp: u64,
    ) -> Result<Option<L2BlockRef>> {
        let block_count = (fd_l2_block_number - finalized_l2_number) as usize;
        let mut l2_blocks = HashMap::new();
        let mut query_blocks = Vec::new();
        let mut largest_non_activated_block = None;

        for i in 0..block_count {
            let block_number = (i as u64) + finalized_l2_number + 1;
            let l2_block = match self.l2_fetcher.l2_block_ref_by_number(block_number) {
                Ok(block) => block,
                Err(e) => {
                    self.emitter.emit(Event::CriticalError(CriticalErrorEvent {
                        err: format!(
                            "failed to check if block {} to {} is finalized on Babylon, could not fetch block {}: {}",
                            finalized_l2_number + 1,
                            fd_l2_block_number,
                            block_number,
                            e
                        )
                    })).await;
                    return Ok(None);
                }
            };

            l2_blocks.insert(block_number, l2_block.clone());

            if l2_block.time < gadget_activated_timestamp {
                largest_non_activated_block = Some(l2_block);
                continue;
            }

            query_blocks.push(Block {
                block_height: l2_block.number,
                block_hash: l2_block.hash.to_string(),
                block_timestamp: l2_block.time,
            });
        }

        if query_blocks.is_empty() {
            return Ok(largest_non_activated_block);
        }

        match self
            .babylon_finality_client
            .query_block_range_babylon_finalized(query_blocks.as_slice())
            .await
        {
            Ok(last_finalized_block_number) => {
                if let Some(number) = last_finalized_block_number {
                    return Ok(l2_blocks.get(&number).cloned());
                }
            }
            Err(e) => {
                self.emitter
                    .emit(Event::CriticalError(CriticalErrorEvent {
                        err: format!(
                            "failed to check if block {} to {} is finalized on Babylon: {}",
                            finalized_l2_number + 1,
                            fd_l2_block_number,
                            e
                        ),
                    }))
                    .await;
            }
        }

        Ok(largest_non_activated_block)
    }

    async fn on_reset(&self) {
        let mut finality_data = self.finality_data.lock().unwrap();
        finality_data.clear();
        *self.tried_finalize_at.lock().unwrap() = 0;
    }

    async fn on_derived_safe_block(&self, l2_safe: L2BlockRef, derived_from: L1BlockRef) {
        let mut finality_data = self.finality_data.lock().unwrap();

        if finality_data.is_empty()
            || finality_data.last().unwrap().l1_block.number < derived_from.number
        {
            if finality_data.len() as u64 >= self.finality_lookback {
                finality_data.drain(0..1);
            }

            finality_data.push(FinalityData {
                l2_block: l2_safe.clone(),
                l1_block: BlockID {
                    number: derived_from.number,
                    hash: derived_from.hash,
                },
            });

            debug!(
                "extended finality-data last_l1 {:?}, last_l2 {:?}",
                finality_data.last().unwrap().l1_block,
                finality_data.last().unwrap().l2_block
            );
        } else {
            let last = finality_data.last_mut().unwrap();
            if last.l2_block != l2_safe {
                last.l2_block = l2_safe;
                debug!(
                    "updated finality-data last_l1 {:?}, last_l2 {:?}",
                    last.l1_block, last.l2_block
                );
            }
        }
    }
}

// Default implementations
#[derive(Default)]
pub struct NoopEmitter;

#[async_trait]
impl EventEmitter for NoopEmitter {
    async fn emit(&self, _event: Event) {}
}

// #[async_trait]
// pub trait FinalizerL1Interface: Send + Sync {
//     async fn l1_block_ref_by_number(&self, number: u64) -> Result<L1BlockRef, Box<dyn std::error::Error + Send + Sync>>;
// }

#[derive(Clone, Debug, Default)]
pub struct FinalizerL1Mock {}

#[async_trait]
impl FinalizerL1Interface for FinalizerL1Mock {
    // async fn l1_block_ref_by_number(&self, number: u64) -> Result<L1BlockRef, Box<dyn std::error::Error + Send + Sync>> {
    async fn l1_block_ref_by_number(&self, _number: u64) -> Result<L1BlockRef> {
        // Implement your gRPC client initialization here
        let mock = L1BlockRef::default();

        Ok(mock)
    }
}
