// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use bitcoin::hashes::Hash;
use move_core_types::account_address::AccountAddress;
use moveos_types::h256::H256;

pub trait IntoAddress {
    fn into_address(self) -> AccountAddress;
}

impl IntoAddress for AccountAddress {
    fn into_address(self) -> AccountAddress {
        self
    }
}

impl IntoAddress for &AccountAddress {
    fn into_address(self) -> AccountAddress {
        *self
    }
}

impl IntoAddress for [u8; AccountAddress::LENGTH] {
    fn into_address(self) -> AccountAddress {
        AccountAddress::new(self)
    }
}

impl IntoAddress for &[u8; AccountAddress::LENGTH] {
    fn into_address(self) -> AccountAddress {
        AccountAddress::new(*self)
    }
}

impl IntoAddress for bitcoin::hashes::sha256d::Hash {
    fn into_address(self) -> AccountAddress {
        AccountAddress::new(self.to_byte_array())
    }
}

impl IntoAddress for bitcoin::BlockHash {
    fn into_address(self) -> AccountAddress {
        AccountAddress::new(self.to_byte_array())
    }
}

impl IntoAddress for bitcoin::Txid {
    fn into_address(self) -> AccountAddress {
        AccountAddress::new(self.to_byte_array())
    }
}

impl IntoAddress for bitcoin::TxMerkleNode {
    fn into_address(self) -> AccountAddress {
        AccountAddress::new(self.to_byte_array())
    }
}

impl IntoAddress for H256 {
    fn into_address(self) -> AccountAddress {
        AccountAddress::new(self.0)
    }
}
