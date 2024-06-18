// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::META_SEQUENCER_INFO_PREFIX_NAME;
use anyhow::Result;
use raw_store::{derive_store, CodecKVStore, StoreInstance};
use rooch_types::sequencer::SequencerInfo;
use std::string::ToString;

pub const SEQUENCER_INFO_KEY: &str = "sequencer_info";

derive_store!(
    SequencerInfoStore,
    String,
    SequencerInfo,
    META_SEQUENCER_INFO_PREFIX_NAME
);

pub trait MetaStore {
    fn get_sequencer_info(&self) -> Result<Option<SequencerInfo>>;

    fn save_sequencer_info(&self, sequencer_info: SequencerInfo) -> Result<()>;
}

#[derive(Clone)]
pub struct MetaDBStore {
    sequencer_info_store: SequencerInfoStore,
}

impl MetaDBStore {
    pub fn new(instance: StoreInstance) -> Self {
        MetaDBStore {
            sequencer_info_store: SequencerInfoStore::new(instance),
        }
    }

    pub fn get_sequencer_info(&self) -> Result<Option<SequencerInfo>> {
        self.sequencer_info_store
            .kv_get(SEQUENCER_INFO_KEY.to_string())
    }

    pub fn save_sequencer_info(&self, sequencer_info: SequencerInfo) -> Result<()> {
        let pre_sequencer_info = self.get_sequencer_info()?;
        if let Some(pre_sequencer_info) = pre_sequencer_info {
            if sequencer_info.last_order != pre_sequencer_info.last_order + 1 {
                return Err(anyhow::anyhow!("Sequencer order is not continuous"));
            }
        }
        self.sequencer_info_store
            .put_sync(SEQUENCER_INFO_KEY.to_string(), sequencer_info)
    }
}
