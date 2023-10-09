// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Utility for converting a Move value to its binary representation in RLP(Recursive Length Prefix)
/// https://ethereum.org/nl/developers/docs/data-structures-and-encoding/rlp/
module moveos_std::rlp{

    native public fun to_bytes<MoveValue>(value: &MoveValue): vector<u8>;
    public(friend) native fun from_bytes<MoveValue>(bytes: &vector<u8>): MoveValue;

}