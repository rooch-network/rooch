// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::h256::H256;
use tiny_keccak::{Hasher, Sha3};

pub const LENGTH: usize = 32;

//TODO use sha2
pub fn sha3_256_of(buffer: &[u8]) -> H256 {
    let mut sha3 = Sha3::v256();
    sha3.update(buffer);
    let mut hash = [0u8; LENGTH];
    sha3.finalize(&mut hash);
    H256(hash)
}
