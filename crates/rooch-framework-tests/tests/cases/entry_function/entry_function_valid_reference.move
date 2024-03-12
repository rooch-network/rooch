//# init --addresses creator=0x42

//# publish
module creator::test {
    use moveos_std::context;

    entry public fun test_entry_function_valid_reference_signer(_: & signer ){
        
    }

    entry public fun test_entry_function_valid_reference_mut_signer(_: &mut signer ){
        
    }

    entry public fun test_entry_function_valid_reference_storage_context(_: & context::Context ){
        
    }

    entry public fun test_entry_function_valid_reference_mut_storage_context(_: &mut context::Context ){
        
    }
}
