// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{ObjectIDVecView, StrView, UnitedAddressView};
use rooch_types::repair::{RepairIndexerParams, RepairIndexerType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub type RepairIndexerTypeView = StrView<RepairIndexerType>;

impl std::fmt::Display for RepairIndexerTypeView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RepairIndexerTypeView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        Ok(StrView(RepairIndexerType::from_str(s)?))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RepairIndexerParamsView {
    /// Repair by owner.
    Owner(UnitedAddressView),
    /// Repair by object ids.
    ObjectId(ObjectIDVecView),
}

impl From<RepairIndexerParamsView> for RepairIndexerParams {
    fn from(repair_params: RepairIndexerParamsView) -> Self {
        match repair_params {
            RepairIndexerParamsView::Owner(owner) => RepairIndexerParams::Owner(owner.into()),
            RepairIndexerParamsView::ObjectId(object_id_vec_view) => {
                RepairIndexerParams::ObjectId(object_id_vec_view.into())
            }
        }
    }
}
