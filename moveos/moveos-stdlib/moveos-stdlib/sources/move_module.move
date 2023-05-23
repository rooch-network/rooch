/// `move_module` provides some basic functions for handle Move module in Move.
module moveos_std::move_module{
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
        abort 0
    }

    // This is a native function that verifies the modules and returns their names
    // This function need to ensure the module's bytecode is valid and the module id is matching the account address.
    public fun verify_modules(_modules: &vector<MoveModule>, _account_address: address): vector<String> {
        //TODO implement native verify modules
        abort 0
    }
}