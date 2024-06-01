// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::multichain_address {

    use rooch_framework::ethereum_address::{Self, ETHAddress};
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};
    use moveos_std::hash::{blake2b256};
    use moveos_std::bcs;

    const ErrorMultiChainIDMismatch: u64 = 1;

    const LENGTH: u64 = 31;

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

    #[data_struct]
    struct MultiChainAddress has copy, store, drop {
        multichain_id: u64,
        raw_address: vector<u8>,
    }

    public fun get_length(): u64 {
        return LENGTH
    }

    public fun new(multichain_id: u64, raw_address: vector<u8>): MultiChainAddress {
        MultiChainAddress {
            multichain_id: multichain_id,
            raw_address: raw_address,
        }
    }

    public fun from_bytes(bytes: vector<u8>): MultiChainAddress {
        bcs::from_bytes<MultiChainAddress>(bytes)
    }

    public fun from_eth(eth_address: ETHAddress): MultiChainAddress {
        MultiChainAddress {
            multichain_id: MULTICHAIN_ID_ETHER,
            raw_address: ethereum_address::into_bytes(eth_address),
        }
    }

    public fun from_bitcoin(bitcoin_address: BitcoinAddress): MultiChainAddress {
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
        assert!(maddress.multichain_id == MULTICHAIN_ID_ROOCH, ErrorMultiChainIDMismatch);
        moveos_std::bcs::to_address(maddress.raw_address)
    }

    public fun into_eth_address(maddress: MultiChainAddress) : ETHAddress {
        assert!(maddress.multichain_id == MULTICHAIN_ID_ETHER, ErrorMultiChainIDMismatch);
        ethereum_address::from_bytes(maddress.raw_address)
    }

    public fun into_bitcoin_address(maddress: MultiChainAddress) : BitcoinAddress {
        assert!(maddress.multichain_id == MULTICHAIN_ID_BITCOIN, ErrorMultiChainIDMismatch);
        bitcoin_address::new(maddress.raw_address)
    }

    /// Mapping from MultiChainAddress to rooch address
    /// If the MultiChainAddress is not rooch address, it will generate a new rooch address based on the MultiChainAddress
    public fun mapping_to_rooch_address(maddress: MultiChainAddress): address {
        if(is_rooch_address(&maddress)) {
            into_rooch_address(maddress)
        }else{
            let hash = blake2b256(&maddress.raw_address);
            moveos_std::bcs::to_address(hash)
        }
    }

    #[test]
    fun test_multi_chain_address_into_rooch_address() {
        let msg = x"00536ffa992491508dca0354e52f32a3a7a679a53a";
        let hashed_msg = blake2b256(&msg);
        let addr = moveos_std::bcs::to_address(hashed_msg);
        let expect_addr = @0x419791e7f82060465cf8c16c8f45ab9930b3a944b18e1df2278807c12ea32c65;
        assert!(expect_addr == addr, 0);
    }
}
