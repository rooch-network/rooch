//# init --addresses creator=0x42

//# publish
module creator::test {
    use moveos_std::tx_context;

    struct Foo has copy, drop {
        x: u64,
    }

    entry public fun test_entry_function_invalid_struct( _foo: Foo ){
        
    }

    entry public fun test_entry_function_invalid_struct_txcontext( _: &tx_context::TxContext ){
        
    }
}
