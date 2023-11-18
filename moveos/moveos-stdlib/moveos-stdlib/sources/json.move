// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::json{

    #[private_generics(T)]
    /// Function to deserialize a type T.
    /// Note the `private_generics` ensure only the `T`'s owner module can call this function
    public fun from_json<T>(bytes: vector<u8>): T {
        native_from_json(bytes)
    }

    native fun native_from_json<T>(bytes: vector<u8>): T;

    #[test_only]
    struct Test has drop, copy {
        balance: u128,
        age: u8,
    }

    #[test]
    // #[expected_failure]
    fun test_from_json() {
        let bytes = b"{\"balance\": \"170141183460469231731687303715884105728\",\"age\":30}";
        let test = from_json<Test>(bytes);
        // let bytes = bcs::to_bytes(&Person{name: 100u128, age: 30u8});
        assert!(test.age == 30u8, 1);
        assert!(test.balance == 170141183460469231731687303715884105728u128, 1);
    }
}