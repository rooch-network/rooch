//# init --addresses creator=0x42

//# publish
module creator::test {
    use std::string;
    use moveos_std::object;

    entry public fun test_entry_function_valid_struct_string(_str: string::String){
        
    }

    entry public fun test_entry_function_valid_struct_object_id(_id: object::ObjectID){
        
    }
}
