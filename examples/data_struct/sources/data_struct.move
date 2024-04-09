// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::data_struct {
    

    #[data_struct]
    struct Inner has copy,drop{
        f_u8: u8,
    }

    #[data_struct]
    struct Outer has copy,drop {
        f_u64: u64,
        f_address: address,
        f_bool: bool,
        f_str: std::string::String,
        f_custom: Inner,
    }

    #[data_struct(T)]
    fun f1<T: drop>(__inner: T): bool{
        false
    }

    fun f2() {
        let inner = Inner {f_u8: 1};
        f1(inner);
    }
}
