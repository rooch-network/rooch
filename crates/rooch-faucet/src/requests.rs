// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_rpc_api::jsonrpc_types::UnitedAddressView;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FaucetRequest {
    pub claimer: UnitedAddressView,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FaucetRequestWithInviter {
    pub claimer: UnitedAddressView,
    pub inviter: UnitedAddressView,
    pub claimer_sign: String,
    pub public_key: String,
}

impl FaucetRequest {
    pub fn recipient(&self) -> UnitedAddressView {
        self.claimer.clone()
    }
}

impl FaucetRequestWithInviter {
    pub fn recipient(&self) -> UnitedAddressView {
        self.claimer.clone()
    }
    pub fn inviter(&self) -> UnitedAddressView {
        self.inviter.clone()
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerifyAndBindingTwitterAccountWithInviter {
    pub tweet_id: String,
    pub inviter: UnitedAddressView,
    pub claimer_sign: String,
    pub public_key: String,
}
