// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::META_SEQUENCER_ORDER_PREFIX_NAME;
use anyhow::Result;
use raw_store::{derive_store, CodecKVStore, StoreInstance};
use rooch_types::sequencer::SequencerOrder;
use std::string::ToString;

pub const SEQUENCER_ORDER_KEY: &str = "sequencer_order";
derive_store!(
    SequencerOrderStore,
    String,
    SequencerOrder,
    META_SEQUENCER_ORDER_PREFIX_NAME
);

pub trait MetaStore {
    fn get_sequencer_order(&self) -> Result<Option<SequencerOrder>>;

    fn save_sequencer_order(&self, sequencer_order: SequencerOrder) -> Result<()>;
}

#[derive(Clone)]
pub struct MetaDBStore {
    sequencr_order_store: SequencerOrderStore,
}

impl MetaDBStore {
    pub fn new(instance: StoreInstance) -> Self {
        MetaDBStore {
            sequencr_order_store: SequencerOrderStore::new(instance),
        }
    }

    pub fn get_sequencer_order(&self) -> Result<Option<SequencerOrder>> {
        self.sequencr_order_store
            .kv_get(SEQUENCER_ORDER_KEY.to_string())
    }

    pub fn save_sequencer_order(&self, sequencer_order: SequencerOrder) -> Result<()> {
        self.sequencr_order_store
            .put_sync(SEQUENCER_ORDER_KEY.to_string(), sequencer_order)
    }
}
