// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::module1 {
    // #[test_only]
    // use rooch_examples::module2::{new_data, Data2};

    struct Data has drop {
        v: u64
    }

    struct Box<T> has drop {
        v: T
    }

    #[private_generics(T)]
    public fun new_box<T, U>(value: T): Box<T> {
        Box { v: value }
    }

    public fun get_box_value<T>(box: &Box<T>): &T {
        &box.v
    }

    #[test]
    fun test1() {
        let data = Data { v: 123 };
        let box = new_box<Data, u64>(data);
        assert!(get_box_value(&box).v == 123, 1000);
    }

    // #[test]
    // fun test2() {
    //     let data2 = new_data(456);
    //     let box2 = new_box<Data2, u64>(data2);
    //     assert!(get_box_value(&box2) == &new_data(456), 2000)
    // }
}
