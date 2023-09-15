// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{
    TransactionExecutionInfoView, TransactionSequenceInfoView, TransactionView,
};
use moveos_types::transaction::TransactionExecutionInfo;
use rooch_types::transaction::{TransactionSequenceInfo, TypedTransaction};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub transaction: TypedTransaction,
    pub sequence_info: TransactionSequenceInfo,
    pub execution_info: TransactionExecutionInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionResultView {
    pub transaction: TransactionView,
    pub sequence_info: TransactionSequenceInfoView,
    pub execution_info: TransactionExecutionInfoView,
}

impl From<TransactionResult> for TransactionResultView {
    fn from(tx: TransactionResult) -> Self {
        Self {
            transaction: tx.transaction.into(),
            sequence_info: tx.sequence_info.into(),
            execution_info: tx.execution_info.into(),
        }
    }
}
