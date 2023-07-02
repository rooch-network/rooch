/// `move_module` provides some basic functions for handle Move module in Move.
module moveos_std::move_module{
    use std::vector;
    use std::string::String;

    struct MoveModule has store, drop {
        byte_codes: vector<u8>,
    }

    public fun new(byte_codes: vector<u8>) : MoveModule {
        MoveModule {
            byte_codes,
        }
    }

    public fun module_name(move_module: &MoveModule): String {
        //TODO implement native module name
        module_name_inner(&move_module.byte_codes)
    }

    // This is a native function that verifies the modules and returns their names
    // This function need to ensure the module's bytecode is valid and the module id is matching the account address.
    public fun verify_modules(modules: &vector<MoveModule>, account_address: address): vector<String> {
        let bytes_vec = vector::empty<vector<u8>>();
        let i = 0u64;
        let len = vector::length(modules);
        while (i < len) {
            vector::push_back(&mut bytes_vec, vector::borrow(modules, i).byte_codes);
            i = i + 1;
        };
        verify_modules_inner(bytes_vec, account_address)
    }

    native fun module_name_inner(byte_codes: &vector<u8>): String;
    native fun verify_modules_inner(modules: vector<vector<u8>>, account_address: address): vector<String>;
}