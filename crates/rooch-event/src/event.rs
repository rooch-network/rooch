// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_types::service_status::ServiceStatus;

#[derive(Default, Clone, Debug)]
pub struct GasUpgradeEvent {}

#[derive(Default, Clone, Debug)]
pub struct ServiceStatusEvent {
    pub status: ServiceStatus,
}
