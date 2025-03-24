// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::moveos_std::decimal_value::DecimalValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub struct DecimalValueView(serde_json::Value);

impl From<DecimalValue> for DecimalValueView {
    fn from(decimal_value: DecimalValue) -> Self {
        DecimalValueView(decimal_value.to_json_value())
    }
}

impl Display for DecimalValueView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
