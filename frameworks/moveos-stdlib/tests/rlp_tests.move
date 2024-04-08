// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module moveos_std::rlp_tests {
    use std::debug;
    use std::vector;
    use std::string::String;
    use moveos_std::rlp;
    
    #[data_struct]
    struct PrimaryStruct has copy, drop {
        a: u8,
        b: u16,
        c: u32,
        d: u64,
        e: u128,
        f: u256,
        me: address,
    }

    #[data_struct]
    struct ChildStruct has copy, drop {
        c: PrimaryStruct,
    }
    
    #[data_struct]
    struct NestedStruct has copy, drop {
        child: ChildStruct,
        v: vector<PrimaryStruct>,
    }

    #[test]
    fun test_basic() {
        let bytes = rlp::to_bytes(&0u8);
        assert!(bytes == x"80", 1);

        let bytes = rlp::to_bytes(&x"0400"); // 1024
        assert!(bytes == x"820400", 2);
        
        let bytes = rlp::to_bytes(&1024u64);
        assert!(bytes == x"820400", 3);

        let bytes = rlp::to_bytes(&b"dog");
        assert!(bytes == x"83646f67", 4);

        let bytes = rlp::to_bytes<String>(&std::string::utf8(b"dog"));
        assert!(bytes == x"c483646f67", 5);

        let v = vector::empty<u32>();
        vector::push_back<u32>(&mut v, 1);
        vector::push_back<u32>(&mut v, 2);
        let bytes = rlp::to_bytes(&v);
        assert!(bytes == x"c20102", 6);

        let bytes = rlp::to_bytes(&@0x42);
        assert!(bytes == x"a00000000000000000000000000000000000000000000000000000000000000042", 7);
    }

    fun primary_struct_instantiation(offset: u8): PrimaryStruct {
        let v = vector::empty<u16>();
        vector::push_back<u16>(&mut v, 1);
        vector::push_back<u16>(&mut v, 2);
        PrimaryStruct { 
            a: 1 + offset, 
            b: 256, 
            c: 3 + (offset as u32), 
            d: 4 + (offset as u64), 
            e: 5 + (offset as u128), 
            f: 6 + (offset as u256), 
            me: @0x42
        }
    }

    fun nested_struct_instantiation(): NestedStruct {
        let v = vector::empty<PrimaryStruct>();
        vector::push_back<PrimaryStruct>(&mut v, primary_struct_instantiation(1));
        let child = ChildStruct { c: primary_struct_instantiation(2)};
        let data = NestedStruct { child: child, v: v  };
        data
    }

    #[test]
    fun test_primary_struct() {
        let data = primary_struct_instantiation(1);
        let bytes = rlp::to_bytes(&data);
        debug::print(&bytes);
        let decoded_data = rlp::from_bytes<PrimaryStruct>(bytes);
        assert!(decoded_data == data, 1);
    }

    #[test]
    fun test_nested_struct() {
        let data = nested_struct_instantiation();
        let bytes = rlp::to_bytes(&data);
        debug::print(&bytes);
        let decoded_data = rlp::from_bytes<NestedStruct>(bytes);
        assert!(decoded_data.child == data.child, 1);
        assert!(vector::borrow(&decoded_data.v, 0) == vector::borrow(&data.v, 0), 2);
    }
}