// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Source from https://github.com/aptos-labs/aptos-core/blob/main/aptos-move/framework/aptos-stdlib/sources/any.move

module moveos_std::any {
    use std::option;
    use std::string::String;    
    use moveos_std::type_info;
    use moveos_std::bcs;

    friend moveos_std::copyable_any;

    /// The type provided for `unpack` is not the same as was given for `pack`.
    const ErrorTypeMismatch: u64 = 1;
    const ErrorInvalidBytes: u64 = 2;

    /// A type which can represent a value of any type. This allows for representation of 'unknown' future
    /// values. For example, to define a resource such that it can be later be extended without breaking
    /// changes one can do
    ///
    /// ```move
    ///   struct Resource {
    ///      field: Type,
    ///      ...
    ///      extension: Option<Any>
    ///   }
    /// ```
    struct Any has drop, store {
        type_name: String,
        data: vector<u8>
    }

    /// Pack a value into the `Any` representation. Because Any can be stored and dropped, this is
    /// also required from `T`.
    public fun pack<T: drop + store>(x: T): Any {
        Any {
            type_name: type_info::type_name<T>(),
            data: bcs::to_bytes(&x)
        }
    }

    /// Unpack a value from the `Any` representation. This aborts if the value has not the expected type `T`.
    public fun unpack<T>(x: Any): T {
        assert!(type_info::type_name<T>() == x.type_name, ErrorTypeMismatch);
        let opt_result = bcs::native_from_bytes<T>(x.data);
        assert!(option::is_some(&opt_result), ErrorInvalidBytes);
        option::destroy_some(opt_result)
    }

    /// Returns the type name of this Any
    public fun type_name(x: &Any): &String {
        &x.type_name
    }

    #[test_only]
    struct S has store, drop { x: u64 }

    #[test]
    fun test_any() {
        assert!(unpack<u64>(pack(22)) == 22, 1);
        assert!(unpack<S>(pack(S { x: 22 })) == S { x: 22 }, 2);
    }
}