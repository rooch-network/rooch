// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::BytesView;
use super::{HumanReadableDisplay, ModuleIdView, StateChangeSetView, StrView};
use crate::jsonrpc_types::event_view::EventView;
use crate::jsonrpc_types::H256View;
use ethers::types::H256;
use move_core_types::vm_status::{AbortLocation, KeptVMStatus};
use moveos_types::transaction::TransactionOutput;
use moveos_types::transaction::{TransactionExecutionInfo, VMErrorInfo};
use rooch_types::transaction::ExecuteTransactionResponse;
use rooch_types::transaction::{authenticator::Authenticator, TransactionSequenceInfo};
use schemars::JsonSchema;
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

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
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
                message: _message,
            } => Self::ExecutionFailure {
                location: location.into(),
                function,
                code_offset,
            },
            KeptVMStatus::MiscellaneousError => Self::MiscellaneousError,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthenticatorView {
    pub auth_validator_id: StrView<u64>,
    pub payload: BytesView,
}

impl From<Authenticator> for AuthenticatorView {
    fn from(authenticator: Authenticator) -> Self {
        Self {
            auth_validator_id: StrView(authenticator.auth_validator_id),
            payload: StrView(authenticator.payload),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionSequenceInfoView {
    pub tx_order: StrView<u64>,
    pub tx_order_signature: BytesView,
    pub tx_accumulator_root: H256View,
    pub tx_timestamp: StrView<u64>,
}

impl TransactionSequenceInfoView {
    fn new(
        tx_order: u64,
        tx_order_signature: Vec<u8>,
        tx_accumulator_root: H256,
        tx_timestamp: u64,
    ) -> Self {
        Self {
            tx_order: StrView(tx_order),
            tx_order_signature: tx_order_signature.into(),
            tx_accumulator_root: tx_accumulator_root.into(),
            tx_timestamp: StrView(tx_timestamp),
        }
    }
}

impl From<TransactionSequenceInfo> for TransactionSequenceInfoView {
    fn from(transaction_sequence_info: TransactionSequenceInfo) -> Self {
        Self {
            tx_order: StrView(transaction_sequence_info.tx_order),
            tx_order_signature: transaction_sequence_info.tx_order_signature.into(),
            tx_accumulator_root: transaction_sequence_info.tx_accumulator_root.into(),
            tx_timestamp: StrView(transaction_sequence_info.tx_timestamp),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransactionExecutionInfoView {
    pub tx_hash: H256View,
    pub state_root: H256View,
    pub event_root: H256View,
    pub gas_used: StrView<u64>,
    pub status: KeptVMStatusView,
}

impl TransactionExecutionInfoView {
    fn new(
        tx_hash: H256,
        state_root: H256,
        event_root: H256,
        gas_used: StrView<u64>,
        status: KeptVMStatusView,
    ) -> Self {
        Self {
            tx_hash: tx_hash.into(),
            state_root: state_root.into(),
            event_root: event_root.into(),
            gas_used,
            status,
        }
    }
}

impl From<TransactionExecutionInfo> for TransactionExecutionInfoView {
    fn from(transaction_execution_info: TransactionExecutionInfo) -> Self {
        Self {
            tx_hash: transaction_execution_info.tx_hash.into(),
            state_root: transaction_execution_info.state_root.into(),
            event_root: transaction_execution_info.event_root.into(),
            gas_used: transaction_execution_info.gas_used.into(),
            status: KeptVMStatusView::from(transaction_execution_info.status),
        }
    }
}

impl HumanReadableDisplay for TransactionExecutionInfoView {
    fn to_human_readable_string(&self, _verbose: bool, indent: usize) -> String {
        format!(
            r#"{indent}Execution info:
{indent}    status: {:?}
{indent}    gas used: {}
{indent}    tx hash: {}
{indent}    state root: {}
{indent}    event root: {}"#,
            self.status,
            self.gas_used,
            self.tx_hash,
            self.state_root,
            self.event_root,
            indent = " ".repeat(indent)
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransactionOutputView {
    pub status: KeptVMStatusView,
    pub changeset: StateChangeSetView,
    pub events: Vec<EventView>,
    pub gas_used: StrView<u64>,
    pub is_upgrade: bool,
}

impl From<TransactionOutput> for TransactionOutputView {
    fn from(tx_output: TransactionOutput) -> Self {
        Self {
            status: tx_output.status.into(),
            changeset: tx_output.changeset.into(),
            events: tx_output
                .events
                .into_iter()
                .map(|event| event.into())
                .collect(),
            gas_used: tx_output.gas_used.into(),
            is_upgrade: tx_output.is_upgrade,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RawTransactionOutputView {
    pub tx_hash: H256View,
    pub state_root: H256View,
    pub status: KeptVMStatusView,
    pub gas_used: StrView<u64>,
    pub is_upgrade: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DryRunTransactionResponseView {
    pub raw_output: RawTransactionOutputView,
    pub vm_error_info: VMErrorInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecuteTransactionResponseView {
    pub sequence_info: TransactionSequenceInfoView,
    pub execution_info: TransactionExecutionInfoView,
    pub output: Option<TransactionOutputView>,
    pub error_info: Option<DryRunTransactionResponseView>,
}

impl ExecuteTransactionResponseView {
    pub fn new_without_output(response: ExecuteTransactionResponse) -> Self {
        Self {
            sequence_info: response.sequence_info.into(),
            execution_info: response.execution_info.into(),
            output: None,
            error_info: None,
        }
    }
}

impl From<DryRunTransactionResponseView> for ExecuteTransactionResponseView {
    fn from(response: DryRunTransactionResponseView) -> Self {
        Self {
            sequence_info: TransactionSequenceInfoView::new(
                u64::MIN,
                Vec::new(),
                H256::random(),
                u64::MIN,
            ),
            execution_info: TransactionExecutionInfoView::new(
                response.raw_output.tx_hash.into(),
                response.raw_output.state_root.into(),
                H256::random(),
                response.raw_output.gas_used,
                response.raw_output.status.clone(),
            ),
            output: None,
            error_info: Some(response),
        }
    }
}

impl From<ExecuteTransactionResponse> for ExecuteTransactionResponseView {
    fn from(response: ExecuteTransactionResponse) -> Self {
        Self {
            sequence_info: response.sequence_info.into(),
            execution_info: response.execution_info.into(),
            output: Some(response.output.into()),
            error_info: None,
        }
    }
}
