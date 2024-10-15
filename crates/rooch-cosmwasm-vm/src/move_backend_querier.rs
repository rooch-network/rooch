// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use cosmwasm_std::{Binary, ContractResult, SystemResult};
use cosmwasm_vm::{BackendResult, GasInfo, Querier};

#[derive(Clone)]
pub struct MoveBackendQuerier;

impl Querier for MoveBackendQuerier {
    fn query_raw(
        &self,
        _request: &[u8],
        gas_limit: u64,
    ) -> BackendResult<SystemResult<ContractResult<Binary>>> {
        // Implement query logic
        (
            Ok(SystemResult::Ok(ContractResult::Ok(Binary::from(vec![])))),
            GasInfo::with_externally_used(gas_limit),
        )
    }
}
