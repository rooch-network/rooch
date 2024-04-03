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

#[cfg(test)]
mod tests {
    use crate::into_address::IntoAddress;
    use bitcoin::hashes::Hash;
    use move_core_types::account_address::AccountAddress;

    #[test]
    fn test_txid_into_address() {
        let addr = AccountAddress::from_hex_literal(
            "0x7fff0feff7702d30165d3c31582fdd3870b1fec34f6cfcb77203b85ecb2cd569",
        )
        .unwrap();
        let txid = bitcoin::Txid::from_byte_array(addr.into());
        //println!("{}", txid);
        //The txid hex string use reverse order
        assert_eq!(
            "69d52ccb5eb80372b7fc6c4fc3feb17038dd2f58313c5d16302d70f7ef0fff7f",
            txid.to_string()
        );

        let txid_zero = bitcoin::Txid::all_zeros();
        let addr_zero = txid_zero.into_address();
        // println!("{}", txid_zero);
        // println!("{}", addr_zero);
        assert_eq!(
            "0000000000000000000000000000000000000000000000000000000000000000",
            addr_zero.to_string()
        );
        assert_eq!(AccountAddress::ZERO, addr_zero);
    }
}
