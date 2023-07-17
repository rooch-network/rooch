module moveos_std::move_action{
    
    use std::error;
    use std::string::{String};
    use std::option::{Self, Option};

    const EInvalidType: u64 = 1;

    struct ScriptCall has store, copy, drop {
        code: vector<u8>,
        //It is hard to implement TypeTag in Move, so we use String instead
        ty_args: vector<String>,
        args: vector<vector<u8>>,
    }

    //TODO define in another module
    struct ModuleId has store, copy, drop {
        module_address: address,
        name: String,
    }

    struct FunctionId has store, copy, drop {
        module_id: ModuleId,
        name: String,
    }
    
    struct FunctionCall has store, copy, drop {
        functionId: FunctionId,
        ty_args: vector<String>,
        args: vector<vector<u8>>,
    }

    struct ModuleBundle has store, copy, drop {
        modules: vector<vector<u8>>,
    }

    struct MoveAction has store, copy, drop {
        script_call: Option<ScriptCall>,
        function_call: Option<FunctionCall>,
        module_bundle: Option<ModuleBundle>,
    }

    public fun is_script_call(move_action: &MoveAction): bool {
        option::is_some(&move_action.script_call)
    }

    public fun into_script_call(move_action: MoveAction): ScriptCall {
        assert!(is_script_call(&move_action), error::invalid_argument(EInvalidType));
        option::extract(&mut move_action.script_call)
    }

    public fun is_function_call(move_action: &MoveAction): bool {
        option::is_some(&move_action.function_call)
    }

    public fun into_function_call(move_action: MoveAction): FunctionCall {
        assert!(is_function_call(&move_action), error::invalid_argument(EInvalidType));
        option::extract(&mut move_action.function_call)
    }

    public fun is_module_bundle(move_action: &MoveAction): bool {
        option::is_some(&move_action.module_bundle)
    }

    public fun into_module_bundle(move_action: MoveAction): ModuleBundle {
        assert!(is_module_bundle(&move_action), error::invalid_argument(EInvalidType));
        option::extract(&mut move_action.module_bundle)
    }

    /// We provide a native function to decode MoveAction from bytes
    /// We can not use the `bcs::from_bytes` directly
    /// because we can not define a `MoveAction` which has same bcs layout with the MoveAction in Rust
    public native fun decode_move_action(bytes: &vector<u8>): MoveAction;
}