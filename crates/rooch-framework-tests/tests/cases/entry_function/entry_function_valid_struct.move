//# init --addresses creator=0x42

//# publish
module creator::test {
    use std::string;
    use std::ascii;
    use moveos_std::context;
    use moveos_std::object_id;

    entry public fun test_entry_function_valid_struct_string(_str: string::String ){
        
    }

    entry public fun test_entry_function_valid_struct_ascii(_ascii_str: ascii::String){

    }

    entry public fun test_entry_function_valid_struct_storage_context(_sctx: &mut context::Context ){
        
    }

    entry public fun test_entry_function_valid_struct_object_id(_id: object_id::ObjectID ){
        
    }
}
