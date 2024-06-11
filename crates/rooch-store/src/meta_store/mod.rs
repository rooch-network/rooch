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
    sequencer_order_store: SequencerOrderStore,
}

impl MetaDBStore {
    pub fn new(instance: StoreInstance) -> Self {
        MetaDBStore {
            sequencer_order_store: SequencerOrderStore::new(instance),
        }
    }

    pub fn get_sequencer_order(&self) -> Result<Option<SequencerOrder>> {
        self.sequencer_order_store
            .kv_get(SEQUENCER_ORDER_KEY.to_string())
    }

    pub fn save_sequencer_order(&self, sequencer_order: SequencerOrder) -> Result<()> {
        let pre_sequencer_order = self.get_sequencer_order()?;
        if let Some(pre_sequencer_order) = pre_sequencer_order {
            if sequencer_order.last_order != pre_sequencer_order.last_order + 1 {
                return Err(anyhow::anyhow!("Sequencer order is not continuous"));
            }
        }
        self.sequencer_order_store
            .put_sync(SEQUENCER_ORDER_KEY.to_string(), sequencer_order)
    }
}
