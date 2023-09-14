// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use moveos_types::h256::H256;
use moveos_types::transaction::TransactionExecutionInfo;
use raw_store::CodecKVStore;

use crate::TRANSACTION_PREFIX_NAME;
use raw_store::derive_store;

derive_store!(
    TransactionDBStore,
    H256,
    TransactionExecutionInfo,
    TRANSACTION_PREFIX_NAME
);

pub trait TransactionStore {
    fn save_tx_execution_info(&self, tx_execution_info: TransactionExecutionInfo) -> Result<()>;
    fn get_tx_execution_info(&self, tx_hash: H256) -> Result<Option<TransactionExecutionInfo>>;
    fn multi_get_tx_execution_infos(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionExecutionInfo>>>;
}

impl TransactionStore for TransactionDBStore {
    fn save_tx_execution_info(&self, tx_execution_info: TransactionExecutionInfo) -> Result<()> {
        self.kv_put(tx_execution_info.tx_hash, tx_execution_info)
    }

    fn get_tx_execution_info(&self, tx_hash: H256) -> Result<Option<TransactionExecutionInfo>> {
        self.kv_get(tx_hash)
    }

    fn multi_get_tx_execution_infos(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<TransactionExecutionInfo>>> {
        self.multiple_get(tx_hashes)
    }
}
