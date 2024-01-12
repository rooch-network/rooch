// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_types::{
    address::{MultiChainAddress, RoochAddress},
    crypto::PublicKey,
    framework::session_key::SessionKey,
    key_struct::EncryptionData,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalSessionKey {
    pub session_key: Option<SessionKey>,
    pub private_key: EncryptionData,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LocalAccount {
    pub address: RoochAddress,
    pub multichain_address: Option<MultiChainAddress>,
    pub public_key: Option<PublicKey>,
    pub has_session_key: bool,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct AddressMapping {
    pub rooch_to_multichain: BTreeMap<RoochAddress, MultiChainAddress>,
    pub multichain_to_rooch: BTreeMap<MultiChainAddress, RoochAddress>,
}
