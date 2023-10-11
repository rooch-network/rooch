// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::module2 {
    struct Data2 has drop {
        v: u64
    }

    public fun new_data(value: u64): Data2 {
        Data2 {
            v: value
        }
    }
}
