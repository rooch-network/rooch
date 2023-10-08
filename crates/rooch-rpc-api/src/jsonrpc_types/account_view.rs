// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::CoinInfoView;
use crate::jsonrpc_types::StrView;
use move_core_types::u256::U256;
use rooch_types::account::BalanceInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::Div;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BalanceInfoView {
    #[serde(flatten)]
    pub coin_info: CoinInfoView,
    pub balance: StrView<U256>,
}

impl BalanceInfoView {
    //TODO implements big decimal calculation for Decimal point display
    pub fn get_balance_show(&self) -> String {
        let balance = U256::div(
            self.balance.0,
            U256::from(10u32.pow(self.coin_info.decimals as u32)),
        );
        balance.to_string()
    }
}

impl From<BalanceInfo> for BalanceInfoView {
    fn from(balance_info: BalanceInfo) -> Self {
        BalanceInfoView {
            coin_info: balance_info.coin_info.into(),
            balance: balance_info.balance.into(),
        }
    }
}
