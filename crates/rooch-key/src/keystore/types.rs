// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_types::{
    address::{BitcoinAddress, RoochAddress},
    crypto::PublicKey,
    framework::session_key::SessionKey,
    key_struct::EncryptionData,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalSessionKey {
    pub session_key: Option<SessionKey>,
    pub private_key: EncryptionData,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LocalAccount {
    pub address: RoochAddress,
    pub bitcoin_address: BitcoinAddress,
    pub public_key: PublicKey,
    pub has_session_key: bool,
}
