// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod natives;

const ERROR_DESCRIPTIONS: &[u8] = include_bytes!("../error_description.errmap");

pub fn error_descriptions() -> &'static [u8] {
    ERROR_DESCRIPTIONS
}
