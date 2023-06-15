//# init --addresses creator=0x42

//# publish
module creator::test {

    entry public fun test_entry_function_return_value_u8():u8{
        0
    }

    entry public fun test_entry_function_return_value_u16():u16{
        0
    }

    entry public fun test_entry_function_return_value_u32():u32{
        0
    }

    entry public fun test_entry_function_return_value_u64():u64{
        0
    }

    entry public fun test_entry_function_return_value_u128():u128{
        0
    }

    entry public fun test_entry_function_return_value_u256():u256{
        0
    }

    entry public fun test_entry_function_return_value_address():address{
        @creator
    }

    entry public fun test_entry_function_return_value_String():std::string::String{
        std::string::utf8(b"")
    }

    struct Foo has copy, drop {
        x: u64,
    }

    entry public fun test_entry_function_return_value_struct():Foo {
        Foo { x: 0 }
    }

    struct FooT<phantom T> has copy, drop {
        x: u64,
    }

    entry public fun test_entry_function_return_value_T<T>(): FooT<T>{
        FooT<T>{ x: 0 }
    }

}
