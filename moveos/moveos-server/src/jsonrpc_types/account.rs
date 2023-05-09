// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{HexEncodedBytes, U64};
use serde::{Deserialize, Serialize};
use moveos_types::{
    move_types::HexEncodedBytes,
};

/// Account data
///
/// A simplified version of the onchain Account resource
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountData {
    pub sequence_number: U64,
    pub authentication_key: HexEncodedBytes,
}
