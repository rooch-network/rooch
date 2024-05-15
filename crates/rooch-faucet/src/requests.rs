// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_types::address::RoochAddress;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FaucetRequest {
    FixedRoochAddressRequest(FixedRoochAddressRequest),
    FixedETHAddressRequest(FixedETHAddressRequest),
    FixedBTCAddressRequest(FixedBTCAddressRequest),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FixedRoochAddressRequest {
    pub recipient: RoochAddress,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FixedETHAddressRequest {
    pub recipient: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FixedBTCAddressRequest {
    pub recipient: String,
}
