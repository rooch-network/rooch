// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_rpc_api::jsonrpc_types::UnitedAddressView;
use rooch_types::address::{BitcoinAddress, EthereumAddress, RoochAddress};
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

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
pub struct FixedRoochAddressRequest {
    pub recipient: RoochAddress,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FixedETHAddressRequest {
    pub recipient: EthereumAddress,
}

#[derive(Serialize, Debug, Clone)]
pub struct FixedBTCAddressRequest {
    pub recipient: BitcoinAddress,
}

impl<'de> Deserialize<'de> for FixedBTCAddressRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TempFixedBTCAddressRequest {
            recipient: String,
        }

        let temp = TempFixedBTCAddressRequest::deserialize(deserializer)?;
        let recipient =
            BitcoinAddress::from_str(&temp.recipient).map_err(serde::de::Error::custom)?;

        Ok(FixedBTCAddressRequest { recipient })
    }
}
