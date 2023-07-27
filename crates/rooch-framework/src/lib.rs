// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod bindings;
pub mod natives;

pub use rooch_types::addresses::*;

const ROOCH_FRAMEWORK_ERROR_DESCRIPTIONS: &[u8] = include_bytes!("../error_description.errmap");

pub fn rooch_framework_error_descriptions() -> &'static [u8] {
    ROOCH_FRAMEWORK_ERROR_DESCRIPTIONS
}
