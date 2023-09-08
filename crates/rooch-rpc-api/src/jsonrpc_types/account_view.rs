// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::jsonrpc_types::{StrView, StructTagView};
use move_core_types::u256::U256;
use rooch_types::account::{AccountInfo, BalanceInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountInfoView {
    pub sequence_number: u64,
    pub balances: Vec<Option<BalanceInfoView>>,
}

impl AccountInfoView {
    pub fn new(sequence_number: u64, balances: Vec<Option<BalanceInfoView>>) -> Self {
        Self {
            sequence_number,
            balances,
        }
    }
}

impl From<AccountInfo> for AccountInfoView {
    fn from(account_info: AccountInfo) -> Self {
        let balances = account_info
            .balances
            .iter()
            .map(|v| v.clone().map(|balance| balance.into()))
            .collect();

        AccountInfoView {
            sequence_number: account_info.sequence_number,
            balances,
        }
    }
}

impl From<AccountInfoView> for AccountInfo {
    fn from(account_info: AccountInfoView) -> Self {
        let balances = account_info
            .balances
            .iter()
            .map(|v| v.clone().map(|balance| balance.into()))
            .collect();
        AccountInfo {
            sequence_number: account_info.sequence_number,
            balances,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BalanceInfoView {
    pub coin_type: StructTagView,
    pub symbol: String,
    pub balance: StrView<U256>,
    pub decimals: u8,
}

impl From<BalanceInfo> for BalanceInfoView {
    fn from(balance_info: BalanceInfo) -> Self {
        BalanceInfoView {
            coin_type: balance_info.coin_type.into(),
            symbol: balance_info.symbol,
            balance: balance_info.balance.into(),
            decimals: balance_info.decimals,
        }
    }
}

impl From<BalanceInfoView> for BalanceInfo {
    fn from(balance_info: BalanceInfoView) -> Self {
        BalanceInfo {
            coin_type: balance_info.coin_type.into(),
            symbol: balance_info.symbol,
            balance: balance_info.balance.into(),
            decimals: balance_info.decimals,
        }
    }
}
