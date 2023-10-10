// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::multichain_address {
    
    use std::error;
    use rooch_framework::ethereum_address::{Self, ETHAddress};
    use rooch_framework::bitcoin_address::{Self, BTCAddress};

    const ErrorMultiChainIDMismatch: u64 = 1;

    //The multichain id standard is defined in [slip-0044](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)
    //Please keep consistent with rust Symbol
    const MULTICHAIN_ID_BITCOIN: u64 = 0;
    const MULTICHAIN_ID_ETHER: u64 = 60;
    const MULTICHAIN_ID_NOSTR: u64 = 1237;
    const MULTICHAIN_ID_ROOCH: u64 = 20230101; // placeholder for MULTICHAIN_ID_ROOCH pending for actual id from slip-0044

    public fun multichain_id_bitcoin(): u64 { MULTICHAIN_ID_BITCOIN }

    public fun multichain_id_ether(): u64 { MULTICHAIN_ID_ETHER }

    public fun multichain_id_nostr(): u64 { MULTICHAIN_ID_NOSTR }

    public fun multichain_id_rooch(): u64 { MULTICHAIN_ID_ROOCH }

    struct MultiChainAddress has copy, store, drop {
        multichain_id: u64,
        raw_address: vector<u8>,
    }

    public fun new(multichain_id: u64, raw_address: vector<u8>): MultiChainAddress {
        MultiChainAddress {
            multichain_id: multichain_id,
            raw_address: raw_address,
        }
    }

    public fun from_eth(eth_address: ETHAddress): MultiChainAddress {
        MultiChainAddress {
            multichain_id: MULTICHAIN_ID_ETHER,
            raw_address: ethereum_address::into_bytes(eth_address),
        }
    }

    public fun from_bitcoin(bitcoin_address: BTCAddress): MultiChainAddress {
        MultiChainAddress {
            multichain_id: MULTICHAIN_ID_BITCOIN,
            raw_address: bitcoin_address::into_bytes(bitcoin_address),
        }
    }

    public fun multichain_id(self: &MultiChainAddress): u64 {
        self.multichain_id
    }

    public fun raw_address(self: &MultiChainAddress): &vector<u8> {
        &self.raw_address
    }

    public fun is_rooch_address(maddress: &MultiChainAddress) : bool{
        maddress.multichain_id == MULTICHAIN_ID_ROOCH
    }

    public fun is_eth_address(maddress: &MultiChainAddress) : bool{
        maddress.multichain_id == MULTICHAIN_ID_ETHER
    }

    public fun is_bitcoin_address(maddress: &MultiChainAddress) : bool{
        maddress.multichain_id == MULTICHAIN_ID_BITCOIN
    }

    public fun into_rooch_address(maddress: MultiChainAddress) : address {
        assert!(maddress.multichain_id == MULTICHAIN_ID_ROOCH, error::invalid_argument(ErrorMultiChainIDMismatch));
        moveos_std::bcs::to_address(maddress.raw_address)
    }

    public fun into_eth_address(maddress: MultiChainAddress) : ETHAddress {
        assert!(maddress.multichain_id == MULTICHAIN_ID_ETHER, error::invalid_argument(ErrorMultiChainIDMismatch));
        ethereum_address::from_bytes(maddress.raw_address)
    }

    public fun into_bitcoin_address(maddress: MultiChainAddress) : BTCAddress {
        assert!(maddress.multichain_id == MULTICHAIN_ID_BITCOIN, error::invalid_argument(ErrorMultiChainIDMismatch));
        bitcoin_address::from_bytes(maddress.raw_address)
    }
}
