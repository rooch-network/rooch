// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::CoinInfoView;
use crate::jsonrpc_types::StrView;
use move_core_types::u256::U256;
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
    pub fn new(coin_info: CoinInfoView, balance: U256) -> Self {
        Self {
            coin_info,
            balance: StrView(balance),
        }
    }

    //TODO implements big decimal calculation for Decimal point display
    pub fn get_balance_show(&self) -> String {
        let balance = U256::div(
            self.balance.0,
            U256::from(10u32.pow(self.coin_info.decimals as u32)),
        );
        balance.to_string()
    }
}
