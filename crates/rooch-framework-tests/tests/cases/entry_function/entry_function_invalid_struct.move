//# init --addresses creator=0x42

//# publish
module creator::test {
    struct Foo has copy, drop {
        x: u64,
    }

    entry public fun test_entry_function_invalid_struct( _foo: Foo ){
        
    }
}
