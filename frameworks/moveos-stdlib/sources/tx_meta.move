// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::tx_meta {

    use std::option::Option;
    
    const MoveActionScriptType: u8 = 0;
    public fun move_action_script_type(): u8 { MoveActionScriptType }
    const MoveActionFunctionType: u8 = 1;
    public fun move_action_function_type(): u8 { MoveActionFunctionType }
    const MoveActionModuleBundleType: u8 = 2;
    public fun move_action_module_bundle_type(): u8 { MoveActionModuleBundleType }

    /// The transaction Meta data
    /// We can not define MoveAction in Move, so we define a simple meta data struct to represent it
    struct TxMeta has store, copy, drop {
        /// The MoveAction type of the current transaction
        action_type: u8,
        /// If the action type is MoveActionFunctionType, this field is the function call meta data
        function_meta: Option<FunctionCallMeta>,
    }

    /// The FunctionCall Meta data
    struct FunctionCallMeta has store, copy, drop {
        module_address: address,
        module_name: std::string::String,
        function_name: std::string::String,
    }

    #[test_only]
    public fun new_function_call_meta(
        module_address: address,
        module_name: std::string::String,
        function_name: std::string::String,
    ): FunctionCallMeta {
        FunctionCallMeta {
            module_address,
            module_name,
            function_name,
        }
    }

    public fun action_type(self: &TxMeta): u8 {
        self.action_type
    }

    public fun is_script_call(self: &TxMeta): bool {
        self.action_type == MoveActionScriptType
    }

    public fun is_function_call(self: &TxMeta): bool {
        self.action_type == MoveActionFunctionType
    }

    public fun is_module_publish(self: &TxMeta): bool {
        self.action_type == MoveActionModuleBundleType
    }

    public fun function_meta(self: &TxMeta): Option<FunctionCallMeta> {
        *&self.function_meta
    }

    public fun function_meta_module_address(function_meta: &FunctionCallMeta): &address {
        &function_meta.module_address
    }

    public fun function_meta_module_name(function_meta: &FunctionCallMeta): &std::string::String {
        &function_meta.module_name
    }

    public fun function_meta_function_name(function_meta: &FunctionCallMeta): &std::string::String {
        &function_meta.function_name
    }

}