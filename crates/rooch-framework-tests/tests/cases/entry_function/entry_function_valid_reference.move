//# init --addresses creator=0x42

//# publish
module creator::test {
    use moveos_std::storage_context;
    use moveos_std::tx_context;


    entry public fun test_entry_function_valid_reference_signer( _: & signer ){
        
    }

    entry public fun test_entry_function_valid_reference_mut_signer( _: &mut signer ){
        
    }

    entry public fun test_entry_function_valid_reference_storage_context( _: & storage_context::StorageContext ){
        
    }

    entry public fun test_entry_function_valid_reference_mut_storage_context( _: &mut storage_context::StorageContext ){
        
    }

    entry public fun test_entry_function_valid_reference_tx_context( _: & tx_context::TxContext ){
        
    }

    entry public fun test_entry_function_valid_reference_mut_tx_context( _: &mut tx_context::TxContext ){
        
    }
    
}
