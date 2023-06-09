// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{ModuleIdView, StrView, TableChangeSetView};
use crate::jsonrpc_types::EventView;
use move_core_types::vm_status::{AbortLocation, KeptVMStatus};
use moveos_types::transaction::TransactionOutput;
use rooch_types::{
    transaction::{
        authenticator::Authenticator, TransactionExecutionInfo, TransactionSequenceInfo,
    },
    H256,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub type AbortLocationView = StrView<AbortLocation>;

impl std::fmt::Display for AbortLocationView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            AbortLocation::Module(module_id) => {
                write!(f, "{}", ModuleIdView::from(module_id.clone()))
            }
            AbortLocation::Script => write!(f, "script"),
        }
    }
}

impl FromStr for AbortLocationView {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "script" => Ok(Self(AbortLocation::Script)),
            _ => {
                let module_id = ModuleIdView::from_str(s)?;
                Ok(Self(AbortLocation::Module(module_id.0)))
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum KeptVMStatusView {
    Executed,
    OutOfGas,
    MoveAbort {
        location: AbortLocationView,
        abort_code: StrView<u64>,
    },
    ExecutionFailure {
        location: AbortLocationView,
        function: u16,
        code_offset: u16,
    },
    MiscellaneousError,
}

impl From<KeptVMStatus> for KeptVMStatusView {
    fn from(vm_status: KeptVMStatus) -> Self {
        match vm_status {
            KeptVMStatus::Executed => Self::Executed,
            KeptVMStatus::OutOfGas => Self::OutOfGas,
            KeptVMStatus::MoveAbort(location, abort_code) => Self::MoveAbort {
                location: location.into(),
                abort_code: StrView(abort_code),
            },
            KeptVMStatus::ExecutionFailure {
                location,
                function,
                code_offset,
            } => Self::ExecutionFailure {
                location: location.into(),
                function,
                code_offset,
            },
            KeptVMStatus::MiscellaneousError => Self::MiscellaneousError,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorView {
    pub scheme: StrView<u64>,
    pub payload: StrView<Vec<u8>>,
}

impl From<Authenticator> for AuthenticatorView {
    fn from(authenticator: Authenticator) -> Self {
        Self {
            scheme: StrView(authenticator.scheme),
            payload: StrView(authenticator.payload),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSequenceInfoView {
    pub tx_order: StrView<u128>,
    pub tx_order_signature: AuthenticatorView,
    pub tx_accumulator_root: H256,
}

impl From<TransactionSequenceInfo> for TransactionSequenceInfoView {
    fn from(transaction_sequence_info: TransactionSequenceInfo) -> Self {
        Self {
            tx_order: StrView(transaction_sequence_info.tx_order),
            tx_order_signature: AuthenticatorView::from(
                transaction_sequence_info.tx_order_signature,
            ),
            tx_accumulator_root: transaction_sequence_info.tx_accumulator_root,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionExecutionInfoView {
    pub tx_hash: H256,
    pub state_root: H256,
    pub event_root: H256,
    pub gas_used: u64,
    pub status: KeptVMStatusView,
}

impl From<TransactionExecutionInfo> for TransactionExecutionInfoView {
    fn from(transaction_execution_info: TransactionExecutionInfo) -> Self {
        Self {
            tx_hash: transaction_execution_info.tx_hash,
            state_root: transaction_execution_info.state_root,
            event_root: transaction_execution_info.event_root,
            gas_used: transaction_execution_info.gas_used,
            status: KeptVMStatusView::from(transaction_execution_info.status),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionOutputView {
    pub status: KeptVMStatusView,
    //TODO The changeset will be removed in the future
    //pub changeset: ChangeSetView,
    pub table_changeset: TableChangeSetView,
    // pub events: Vec<Event>,
    pub events: Vec<EventView>,
    pub gas_used: u64,
}

impl From<TransactionOutput> for TransactionOutputView {
    fn from(tx_output: TransactionOutput) -> Self {
        Self {
            status: tx_output.status.into(),
            table_changeset: tx_output.table_changeset.into(),
            // events: tx_output.events.into_iter().into(),
            events: tx_output
                .events
                .iter()
                .map(|event| event.clone().into())
                .collect(),
            gas_used: tx_output.gas_used,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecuteTransactionResponse {
    pub sequence_info: TransactionSequenceInfo,
    pub execution_info: TransactionExecutionInfo,
    pub output: TransactionOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteTransactionResponseView {
    pub sequence_info: TransactionSequenceInfoView,
    pub execution_info: TransactionExecutionInfoView,
    pub output: TransactionOutputView,
}

impl From<ExecuteTransactionResponse> for ExecuteTransactionResponseView {
    fn from(response: ExecuteTransactionResponse) -> Self {
        Self {
            sequence_info: response.sequence_info.into(),
            execution_info: response.execution_info.into(),
            output: response.output.into(),
        }
    }
}
