// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

mod adapter;
mod avail;
mod celestia;
mod manager;
mod opendal;

pub use self::manager::OpenDABackendManager;
use rooch_config::da_config::OpenDAScheme;

pub fn derive_identifier(scheme: OpenDAScheme) -> String {
    format!("openda-{}", scheme)
}
