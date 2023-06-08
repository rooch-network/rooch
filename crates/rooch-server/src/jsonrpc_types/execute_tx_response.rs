// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_types::transaction::{TransactionExecutionInfo, TransactionSequenceInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteTransactionResponse {
    pub sequence_info: TransactionSequenceInfo,
    pub execution_info: TransactionExecutionInfo,
}
