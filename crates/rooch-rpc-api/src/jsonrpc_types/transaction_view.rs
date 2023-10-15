// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    TransactionExecutionInfoView, TransactionInfoView, TransactionSequenceInfoView,
};
use moveos_types::transaction::TransactionExecutionInfo;
use rooch_types::transaction::{TransactionSequenceInfo, TypedTransaction};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Transaction with sequence info and execution info.
#[derive(Debug, Clone)]
pub struct TransactionWithInfo {
    pub transaction: TypedTransaction,
    pub sequence_info: TransactionSequenceInfo,
    pub execution_info: TransactionExecutionInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionViewResult {
    pub transaction: TransactionInfoView,
    pub sequence_info: TransactionSequenceInfoView,
    pub execution_info: TransactionExecutionInfoView,
}

impl From<TransactionWithInfo> for TransactionViewResult {
    fn from(tx: TransactionWithInfo) -> Self {
        Self {
            transaction: tx.transaction.into(),
            sequence_info: tx.sequence_info.into(),
            execution_info: tx.execution_info.into(),
        }
    }
}
