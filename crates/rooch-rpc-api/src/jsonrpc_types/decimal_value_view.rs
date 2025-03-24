// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::moveos_std::decimal_value::DecimalValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use std::fmt::Display;

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize, Eq, PartialEq)]
pub struct DecimalValueView(#[schemars(with = "Number")] DecimalValue);

impl From<DecimalValue> for DecimalValueView {
    fn from(decimal_value: DecimalValue) -> Self {
        DecimalValueView(decimal_value)
    }
}

impl Display for DecimalValueView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
