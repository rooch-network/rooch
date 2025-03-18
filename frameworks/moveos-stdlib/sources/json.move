// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::json{
    
    use std::string::String;
    use std::option::{Self, Option};
    use moveos_std::simple_map::{Self, SimpleMap};

    /// Error if the `T` is not a struct
    const ErrorTypeNotMatch: u64 = 1;
    /// Error if the json string is invalid
    const ErrorInvalidJSONString: u64 = 2;

    #[data_struct(T)]
    /// Function to deserialize a type T.
    /// The u128 and u256 types must be json String type instead of Number type
    public fun from_json<T: copy >(json_str: vector<u8>): T {
        let opt_result = native_from_json(json_str);
        assert!(option::is_some(&opt_result), ErrorInvalidJSONString);
        option::destroy_some(opt_result)
    }

    #[data_struct(T)]
    /// Function to deserialize a type T.
    /// If the json string is invalid, it will return None
    public fun from_json_option<T: copy >(json_str: vector<u8>): Option<T> {
        native_from_json(json_str)
    }

    /// Parse a json object string to a SimpleMap
    /// If the json string is invalid, it will return an empty SimpleMap
    /// If the field type is primitive type, it will be parsed to String, array or object will be parsed to json string
    public fun to_map(json_str: vector<u8>): SimpleMap<String,String>{
        let opt_result = native_from_json<SimpleMap<String,String>>(json_str);
        if(option::is_none(&opt_result)){
            option::destroy_none(opt_result);
            return simple_map::new()
        };
        option::destroy_some(opt_result)
    }

    /// Serialize a value of type T to JSON string bytes.
    public fun to_json<T>(value: &T): vector<u8> {
        native_to_json(value)
    }

    native fun native_from_json<T>(json_str: vector<u8>): Option<T>;
    native fun native_to_json<T>(value: &T): vector<u8>;

    #[test_only]
    use std::vector;
    #[test_only]
    use std::string;
    #[test_only]
    use moveos_std::decimal_value;
    #[test_only]
    use moveos_std::decimal_value::DecimalValue;
    #[test_only]
    use moveos_std::object;
    #[test_only]
    use moveos_std::object::ObjectID;

    #[test_only]
    #[data_struct]
    struct Inner has copy, drop, store {
        value: u64,
    }
    #[test_only]
    #[data_struct]
    struct Test has copy, drop, store {
        balance: u128,
        utf8_string: std::string::String,
        age: u8,
        inner: Inner,
        bytes: vector<u8>, 
        inner_array: vector<Inner>,
        account: address,
    }

    #[test]
    fun test_from_json() {
        let json_str = b"{\"balance\": \"170141183460469231731687303715884105728\",\"utf8_string\":\"rooch.network\",\"age\":30,\"inner\":{\"value\":100},\"bytes\":[3,3,2,1],\"inner_array\":[{\"value\":101}],\"account\":\"rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqppq6exstd\"}";
        let obj = from_json<Test>(json_str);
        
        assert!(obj.balance == 170141183460469231731687303715884105728u128, 1);
        assert!(obj.age == 30u8, 2);
        assert!(obj.inner.value == 100u64, 3);

        // check bytes
        assert!(vector::length(&obj.bytes) == 4, 4);
        assert!(vector::borrow(&obj.bytes, 0) == &3u8, 5);
        assert!(vector::borrow(&obj.bytes, 1) == &3u8, 6);
        assert!(vector::borrow(&obj.bytes, 2) == &2u8, 7);
        assert!(vector::borrow(&obj.bytes, 3) == &1u8, 8);

        // check inner_array
        assert!(vector::length(&obj.inner_array) == 1, 9);
        assert!(vector::borrow(&obj.inner_array, 0).value == 101u64, 10);

        // check account
        assert!(obj.account == @0x42, 11);
    }

    #[test]
    fun test_to_map(){
        let json_str = b"{\"balance\": \"170141183460469231731687303715884105728\",\"string\":\"rooch.network\",\"age\":30,\"bool_value\": true, \"null_value\": null, \"account\":\"0x42\", \"inner\":{\"value\":100},\"bytes\":[3,3,2,1],\"inner_array\":[{\"value\":101}]}";
        let map = to_map(json_str);
        assert!(simple_map::borrow(&map, &string::utf8(b"balance")) == &string::utf8(b"170141183460469231731687303715884105728"), 1);
        assert!(simple_map::borrow(&map, &string::utf8(b"string")) == &string::utf8(b"rooch.network"), 2);
        assert!(simple_map::borrow(&map, &string::utf8(b"age")) == &string::utf8(b"30"), 4);
        assert!(simple_map::borrow(&map, &string::utf8(b"bool_value")) == &string::utf8(b"true"), 5);
        assert!(simple_map::borrow(&map, &string::utf8(b"null_value")) == &string::utf8(b"null"), 6);
        assert!(simple_map::borrow(&map, &string::utf8(b"account")) == &string::utf8(b"0x42"), 7);
        assert!(simple_map::borrow(&map, &string::utf8(b"inner")) == &string::utf8(b"{\"value\":100}"), 8);
        assert!(simple_map::borrow(&map, &string::utf8(b"bytes")) == &string::utf8(b"[3,3,2,1]"), 9);
        assert!(simple_map::borrow(&map, &string::utf8(b"inner_array")) == &string::utf8(b"[{\"value\":101}]"), 10);
    }

    #[test]
    fun test_invalid_json_to_map(){
        let invalid_json = b"abcd";
        let map = to_map(invalid_json);
        assert!(simple_map::length(&map) == 0, 1);
    }

    #[test]
    fun test_invalid_json_from_json(){
        let invalid_json = b"abcd";
        let obj = from_json_option<Test>(invalid_json);
        assert!(option::is_none(&obj), 1);
    }

    #[test]
    fun test_to_json_basic_types() {
        // Test u8
        let u8_value = 255u8;
        let u8_json = to_json(&u8_value);
        assert!(string::utf8(u8_json) == string::utf8(b"255"), 1);

        // Test u64
        let u64_value = 18446744073709551615u64;
        let u64_json = to_json(&u64_value);
        assert!(string::utf8(u64_json) == string::utf8(b"18446744073709551615"), 2);

        // Test u128
        let u128_value = 340282366920938463463374607431768211455u128;
        let u128_json = to_json(&u128_value);
        assert!(string::utf8(u128_json) == string::utf8(b"\"340282366920938463463374607431768211455\""), 3);

        // Test String
        let string_value = string::utf8(b"rooch.network");
        let string_json = to_json(&string_value);
        assert!(string::utf8(string_json) == string::utf8(b"\"rooch.network\""), 5);

        // Test vector<u8>
        let bytes_value = vector::empty<u8>();
        vector::push_back(&mut bytes_value, 1u8);
        vector::push_back(&mut bytes_value, 2u8);
        vector::push_back(&mut bytes_value, 3u8);
        let bytes_json = to_json(&bytes_value);
        assert!(string::utf8(bytes_json) == string::utf8(b"[1,2,3]"), 6);
    }

    #[test_only]
    struct InnerStruct has copy, drop {
        inner_value: u64
    }

    #[test_only]
    struct OuterStruct has copy, drop {
        outer_value: u64,
        inner_struct: InnerStruct
    }

    #[test_only]
    struct SimpleStruct has copy, drop, store {
        value: u64
    }

    #[test_only]
    struct TestStruct has key {
        count: u64,
    }

    #[test_only]
    #[data_struct]
    struct TestAddressStruct has key,copy,drop {
        addr: address,
    }

    #[test]
    fun test_to_json_composite_types() {
        let inner_struct = InnerStruct { inner_value: 42 };
        let outer_struct = OuterStruct { outer_value: 100, inner_struct: inner_struct };
        let outer_json = to_json(&outer_struct);
        assert!(string::utf8(outer_json) == string::utf8(b"{\"outer_value\":100,\"inner_struct\":{\"inner_value\":42}}"), 1);

        // Test array of structs
        let struct_array = vector::empty<SimpleStruct>();
        vector::push_back(&mut struct_array, SimpleStruct { value: 1 });
        vector::push_back(&mut struct_array, SimpleStruct { value: 2 });
        vector::push_back(&mut struct_array, SimpleStruct { value: 3 });
        let array_json = to_json(&struct_array);
        assert!(string::utf8(array_json) == string::utf8(b"[{\"value\":1},{\"value\":2},{\"value\":3}]"), 2);
    }

    #[test_only]
    struct StructWithEmptyString has copy, drop {
        value: u64,
        empty_string: String
    }

    #[test]
    fun test_to_json_boundary_conditions() {
        // Test empty array
        let empty_array = vector::empty<u8>();
        let empty_array_json = to_json(&empty_array);
        assert!(string::utf8(empty_array_json) == string::utf8(b"[]"), 1);

        // Test struct with empty string
        let empty_string_struct = StructWithEmptyString {
            value: 0,
            empty_string: string::utf8(b"")
        };
        let empty_string_json = to_json(&empty_string_struct);
        assert!(string::utf8(empty_string_json) == string::utf8(b"{\"value\":0,\"empty_string\":\"\"}"), 2);
    }

    #[test]
    fun test_to_json_boolean_and_null() {
        // Test boolean values
        let bool_true = true;
        let bool_true_json = to_json(&bool_true);
        assert!(string::utf8(bool_true_json) == string::utf8(b"true"), 1);

        let bool_false = false;
        let bool_false_json = to_json(&bool_false);
        assert!(string::utf8(bool_false_json) == string::utf8(b"false"), 2);

        // Test null value
        let null_value = option::none<u64>();
        let null_json = to_json(&null_value);
        assert!(string::utf8(null_json) == string::utf8(b"null"), 3);
    }

    #[test]
    fun test_to_json_composite_all() {
        let inner = Inner { value: 100 };
        let inner_array = vector::empty<Inner>();
        vector::push_back(&mut inner_array, Inner { value: 101 });

        let test_obj = Test {
            balance: 170141183460469231731687303715884105728u128,
            utf8_string: string::utf8(b"rooch.network"),
            age: 30u8,
            inner: inner,
            bytes: vector::empty<u8>(),
            inner_array: inner_array,
            account: @0x42,
        };

        let json_str = to_json(&test_obj);

        let map = to_map(json_str);
        assert!(simple_map::borrow(&map, &string::utf8(b"balance")) == &string::utf8(b"170141183460469231731687303715884105728"), 1);
        assert!(simple_map::borrow(&map, &string::utf8(b"utf8_string")) == &string::utf8(b"rooch.network"), 2);
        assert!(simple_map::borrow(&map, &string::utf8(b"age")) == &string::utf8(b"30"), 3);
        assert!(simple_map::borrow(&map, &string::utf8(b"inner")) == &string::utf8(b"{\"value\":100}"), 4);
        assert!(simple_map::borrow(&map, &string::utf8(b"bytes")) == &string::utf8(b"[]"), 5);
        assert!(simple_map::borrow(&map, &string::utf8(b"inner_array")) == &string::utf8(b"[{\"value\":101}]"), 6);
        // rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqppq6exstd is the bech32 address of 0x42
        assert!(simple_map::borrow(&map, &string::utf8(b"account")) == &string::utf8(b"rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqppq6exstd"), 7);
    }

    #[test]
    fun test_json_object() {
        let root_object_id = object::new_object_id_for_test(vector[]);
        let parent_id = SimpleStruct { value: 1 };
        let id = SimpleStruct { value: 10 };
        let parent_object_id = object::custom_object_id<SimpleStruct, TestStruct>(parent_id);
        let object_id = object::custom_object_id_with_parent<SimpleStruct, TestStruct>(parent_object_id, id);

        let root_object_id_json = to_json(&root_object_id);
        let object_id_json = to_json(&object_id);
        let parent_object_id_json = to_json(&parent_object_id);

        let from_root_object_id = from_json<ObjectID>(root_object_id_json);
        let from_object_id = from_json<ObjectID>(object_id_json);
        let from_parent_object_id = from_json<ObjectID>(parent_object_id_json);
        assert!(root_object_id == from_root_object_id, 1);
        assert!(object_id == from_object_id, 2);
        assert!(parent_object_id == from_parent_object_id, 3);
        std::debug::print(&string::utf8(object_id_json));
        std::debug::print(&string::utf8(parent_object_id_json));

        // ensure object id to json result is consistent with ObjectID.tostring()
        assert!(&string::utf8(object_id_json) == &string::utf8(b"\"0xa7afe75c4f3a7631191905601f4396b25dde044539807de65ed4fc7358dbd98e922b7bfcb1937ef58a03e80216493fff916d18cddc747b7a1fb93ce631ee9c62\""), 4);
        assert!(&string::utf8(parent_object_id_json) == &string::utf8(b"\"0xa7afe75c4f3a7631191905601f4396b25dde044539807de65ed4fc7358dbd98e\""), 5);
        let str_json = to_json(&string::utf8(b"abc"));
        assert!(&string::utf8(str_json) == &string::utf8(b"\"abc\""), 6);
    }

    #[test]
    fun test_json_decimal_value() {
        // ("1.000000", DecimalValue { value: U256::from(1u64), decimal: 0 }),
        let decimal_value = decimal_value::new(1000000, 6);
        let decimal_value_json = to_json(&decimal_value);
        //std::debug::print(&string::utf8(decimal_value_json));
        let from_decimal_value = from_json<DecimalValue>(decimal_value_json);
        //std::debug::print(&from_decimal_value);
        assert!(decimal_value::is_equal(&decimal_value,&from_decimal_value), 1);

        // ("1234.567", DecimalValue { value: U256::from(1234567u64), decimal: 3 }),
        let decimal_value2 = decimal_value::new(1234567, 3);
        let decimal_value_json2 = to_json(&decimal_value2);
        let from_decimal_value2 = from_json<DecimalValue>(decimal_value_json2);
        assert!(decimal_value2 == from_decimal_value2, 2);

        // ("0.123", DecimalValue { value: U256::from(123u64), decimal: 3 }),
        let decimal_value3 = decimal_value::new(123, 3);
        let decimal_value_json3 = to_json(&decimal_value3);
        let from_decimal_value3 = from_json<DecimalValue>(decimal_value_json3);
        assert!(decimal_value3 == from_decimal_value3, 3);

        // ("25", DecimalValue { value: U256::from(25u64), decimal: 0 }),
        let decimal_value4 = decimal_value::new(25, 0);
        let decimal_value_json4 = to_json(&decimal_value4);
        let from_decimal_value4 = from_json<DecimalValue>(decimal_value_json4);
        assert!(decimal_value4 == from_decimal_value4, 4);
    }

    #[test]
    fun test_json_decimal_value_number_and_string() {
        let decimal_value = from_json<DecimalValue>(b"\"1.1\"");
        let decimal_value2 = from_json<DecimalValue>(b"1.1");
        assert!(decimal_value == decimal_value2, 1);
        let decimal_json = to_json(&decimal_value);
        assert!(decimal_json == b"1.1", 2);
    }

    #[test]
    fun test_json_decimal_value_too_bug(){
        //u128 max
        let decimal_value = decimal_value::new(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,2);
        let decimal_value_json = to_json(&decimal_value);
        //std::debug::print(&string::utf8(decimal_value_json));
        //The value is too big to be represented in float, so it will be represented in string
        assert!(decimal_value_json == b"\"3402823669209384634633746074317682114.55\"", 2);
    }

    #[test]
    fun test_json_address_in_struct() {
        let _test_address_bech32 = b"";
        let test_struct = TestAddressStruct {
            addr: @0xa7afe75c4f3a7631191905601f4396b25dde044539807de65ed4fc7358dbd98e
        };
        let test_struct_json = to_json(&test_struct);
        let from_test_struct = from_json<TestAddressStruct>(test_struct_json);
        assert!(test_struct == from_test_struct, 1);
    }

    //The json from address can be either hex or bech32
    //But the json to address will always be bech32
    #[test]
    fun test_json_address_hex_and_bech32(){
        let addr = @0x42;
        let test_address_struct = from_json<TestAddressStruct>(b"{\"addr\":\"rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqppq6exstd\"}");
        let test_address_struct2 = from_json<TestAddressStruct>(b"{\"addr\":\"0x42\"}");
        assert!(addr == test_address_struct.addr, 1);
        assert!(addr == test_address_struct2.addr, 2);

        let address_json = to_json(&addr);
        assert!(address_json == b"\"rooch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqppq6exstd\"", 3);
    }
}