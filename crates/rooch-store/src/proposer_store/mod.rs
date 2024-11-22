// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::PROPOSER_LAST_BLOCK_COLUMN_FAMILY_NAME;
use raw_store::{derive_store, CodecKVStore};

pub const PROPOSER_LAST_BLOCK_KEY: &str = "proposer_last_block";

derive_store!(
    ProposerLastBlockStore,
    String,
    u128,
    PROPOSER_LAST_BLOCK_COLUMN_FAMILY_NAME
);

pub trait ProposerStore {
    fn get_last_proposed(&self) -> anyhow::Result<Option<u128>>;
    fn set_last_proposed(&self, block_number: u128) -> anyhow::Result<()>;
    fn clear_last_proposed(&self) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct ProposerDBStore {
    last_block_store: ProposerLastBlockStore,
}

impl ProposerDBStore {
    pub fn new(instance: raw_store::StoreInstance) -> Self {
        ProposerDBStore {
            last_block_store: ProposerLastBlockStore::new(instance),
        }
    }
}

impl ProposerStore for ProposerDBStore {
    fn get_last_proposed(&self) -> anyhow::Result<Option<u128>> {
        self.last_block_store
            .kv_get(PROPOSER_LAST_BLOCK_KEY.to_string())
    }

    fn set_last_proposed(&self, block_number: u128) -> anyhow::Result<()> {
        self.last_block_store
            .put_sync(PROPOSER_LAST_BLOCK_KEY.to_string(), block_number)
    }

    fn clear_last_proposed(&self) -> anyhow::Result<()> {
        self.last_block_store
            .remove(PROPOSER_LAST_BLOCK_KEY.to_string())
    }
}
