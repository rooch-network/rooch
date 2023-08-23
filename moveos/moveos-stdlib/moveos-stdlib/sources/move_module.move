/// `move_module` provides some basic functions for handle Move module in Move.
module moveos_std::move_module{
    use std::vector;
    use std::string::String;

    friend moveos_std::account_storage;
    
    struct MoveModule has store, drop {
        byte_codes: vector<u8>,
    }

    public fun new(byte_codes: vector<u8>) : MoveModule {
        MoveModule {
            byte_codes,
        }
    }

    public fun module_name(move_module: &MoveModule): String {
        module_name_inner(&move_module.byte_codes)
    }

    /// Verifies the modules and returns their names
    /// This function need to ensure the module's bytecode is valid and the module id is matching the account address.
    /// Return
    ///  The first vector is the module names of all the modules.
    ///  The second vector is the module names of the modules with init function.
    public fun verify_modules(modules: &vector<MoveModule>, account_address: address): (vector<String>, vector<String>) {
        let bytes_vec = vector::empty<vector<u8>>();
        let i = 0u64;
        let len = vector::length(modules);
        while (i < len) {
            vector::push_back(&mut bytes_vec, vector::borrow(modules, i).byte_codes);
            i = i + 1;
        };
        verify_modules_inner(bytes_vec, account_address)
    }

    /// Check module compatibility when upgrading
    /// Abort if the new module is not compatible with the old module.
    public fun check_comatibility(new_module: &MoveModule, old_module: &MoveModule) {
        check_compatibililty_inner(new_module.byte_codes, old_module.byte_codes);
    }

    native fun module_name_inner(byte_codes: &vector<u8>): String;
    /// Native function that verifies the modules and returns their names and 
    /// names of the modules with init function
    /// Return
    ///  The first vector is the module names of all the modules.
    ///  The second vector is the module names of the modules with init function.
    native fun verify_modules_inner(modules: vector<vector<u8>>, account_address: address): (vector<String>, vector<String>);
    /// Request to call the init functions of the given modules
    /// module_names: names of modules which have a init function
    /// account_address: address of all the modules
    native public(friend) fun request_init_functions(module_names: vector<String>, account_address: address);

    native fun check_compatibililty_inner(new_bytecodes: vector<u8>, old_bytecodes: vector<u8>);
}