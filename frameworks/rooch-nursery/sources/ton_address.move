// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::ton_address {

    struct TonAddress has store, copy, drop{
        is_nagative: bool,
        //The workchain in TonAddress is i32, but No i32 in Move
        //So we use u32 instead, and use `is_nagative` to represent the sign
        workchain: u32,
        hash_part: address,
    }

}