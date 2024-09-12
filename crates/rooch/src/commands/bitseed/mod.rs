// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod commands;
pub mod generator;
pub mod inscribe;
pub mod inscription;
pub mod operation;
pub mod sft;

pub const PROTOCOL: &str = "bitseed";
pub const METADATA_OP: &str = "op";
pub const METADATA_TICK: &str = "tick";
pub const METADATA_AMOUNT: &str = "amount";
pub const METADATA_ATTRIBUTES: &str = "attributes";
pub const GENERATOR_TICK: &str = "generator";
