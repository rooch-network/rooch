// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::jsonrpc_types::StrView;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Bip9SoftforkStatusView {
    Defined,
    Started,
    LockedIn,
    Active,
    Failed,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct Bip9SoftforkStatisticsView {
    pub period: StrView<u32>,
    pub threshold: Option<StrView<u32>>,
    pub elapsed: StrView<u32>,
    pub count: StrView<u32>,
    pub possible: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct Bip9SoftforkInfoView {
    pub status: Bip9SoftforkStatusView,
    pub bit: Option<StrView<u8>>,
    // Can be -1 for 0.18.x inactive ones.
    pub start_time: StrView<i64>,
    pub timeout: StrView<u64>,
    pub since: StrView<u32>,
    pub statistics: Option<Bip9SoftforkStatisticsView>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SoftforkTypeView {
    Buried,
    Bip9,
    #[serde(other)]
    Other,
}

/// Status of a softfork
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct SoftforkView {
    #[serde(rename = "type")]
    pub type_: SoftforkTypeView,
    pub bip9: Option<Bip9SoftforkInfoView>,
    pub height: Option<StrView<u32>>,
    pub active: bool,
}
