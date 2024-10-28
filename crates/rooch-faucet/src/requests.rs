// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_rpc_api::jsonrpc_types::UnitedAddressView;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FaucetRequest {
    pub claimer: UnitedAddressView,
}

impl FaucetRequest {
    pub fn recipient(&self) -> UnitedAddressView {
        self.claimer.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FetchTweetRequest {
    pub tweet_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerifyAndBindingTwitterAccountRequest {
    pub tweet_id: String,
}
