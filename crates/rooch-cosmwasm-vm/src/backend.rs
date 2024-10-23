// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use cosmwasm_vm::Backend;

use super::move_backend_api::MoveBackendApi;
use super::move_backend_querier::MoveBackendQuerier;
use super::proxy_storage::ProxyStorage;

pub fn build_move_proxy_backend() -> Backend<MoveBackendApi, ProxyStorage, MoveBackendQuerier> {
    Backend {
        api: MoveBackendApi,
        storage: ProxyStorage::new(),
        querier: MoveBackendQuerier,
    }
}
