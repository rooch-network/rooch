// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    TransactionExecutionInfoView, TransactionSequenceInfoView, TransactionView,
};
use rooch_types::transaction::TransactionWithInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionWithInfoView {
    pub transaction: TransactionView,
    pub sequence_info: TransactionSequenceInfoView,
    pub execution_info: TransactionExecutionInfoView,
}

impl From<TransactionWithInfo> for TransactionWithInfoView {
    fn from(tx: TransactionWithInfo) -> Self {
        Self {
            transaction: tx.transaction.into(),
            sequence_info: tx.sequence_info.into(),
            execution_info: tx.execution_info.into(),
        }
    }
}
