// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use cosmwasm_vm::{BackendApi, BackendError, BackendResult, GasInfo};

#[derive(Clone)]
pub struct MoveBackendApi;

impl BackendApi for MoveBackendApi {
    fn addr_validate(&self, _human: &str) -> BackendResult<()> {
        // Implement address validation logic
        (Ok(()), GasInfo::new(1, 0))
    }

    fn addr_canonicalize(&self, human: &str) -> BackendResult<Vec<u8>> {
        // Implement address canonicalization logic
        (Ok(human.as_bytes().to_vec()), GasInfo::new(1, 0))
    }

    fn addr_humanize(&self, canonical: &[u8]) -> BackendResult<String> {
        // Implement address humanization logic
        match String::from_utf8(canonical.to_vec()) {
            Ok(human) => (Ok(human), GasInfo::new(1, 0)),
            Err(_) => (Err(BackendError::InvalidUtf8 {}), GasInfo::new(1, 0)),
        }
    }
}
