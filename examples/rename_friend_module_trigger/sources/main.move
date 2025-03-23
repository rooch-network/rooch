// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::rename_friend {

    friend rooch_examples::dep1;

    public(friend) fun value(): u64 {
        123
    }
}
