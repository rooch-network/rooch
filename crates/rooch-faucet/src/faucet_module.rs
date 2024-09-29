// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, language_storage::ModuleId,
    value::MoveValue,
};
use moveos_types::{
    move_types::FunctionId, moveos_std::object::ObjectID, state::MoveState,
    transaction::FunctionCall,
};

pub const MODULE_NAME: &IdentStr = ident_str!("gas_faucet");

pub const CHECK_CLAIM_FUNCTION: &IdentStr = ident_str!("check_claim");

pub const CLAIM_FUNCTION: &IdentStr = ident_str!("claim");

pub fn check_claim_function_call(
    module_address: AccountAddress,
    faucet_object_id: ObjectID,
    claimer: AccountAddress,
    utxo_ids: Vec<ObjectID>,
) -> FunctionCall {
    FunctionCall {
        function_id: FunctionId::new(
            ModuleId::new(module_address, MODULE_NAME.to_owned()),
            CHECK_CLAIM_FUNCTION.to_owned(),
        ),
        ty_args: vec![],
        args: vec![
            faucet_object_id.to_move_value().simple_serialize().unwrap(),
            MoveValue::Address(claimer).simple_serialize().unwrap(),
            MoveValue::Vector(utxo_ids.iter().map(|id| id.to_move_value()).collect())
                .simple_serialize()
                .unwrap(),
        ],
    }
}

pub fn claim_function_call(
    module_address: AccountAddress,
    faucet_object_id: ObjectID,
    claimer: AccountAddress,
    utxo_ids: Vec<ObjectID>,
) -> FunctionCall {
    FunctionCall {
        function_id: FunctionId::new(
            ModuleId::new(module_address, MODULE_NAME.to_owned()),
            CLAIM_FUNCTION.to_owned(),
        ),
        ty_args: vec![],
        args: vec![
            faucet_object_id.to_move_value().simple_serialize().unwrap(),
            MoveValue::Address(claimer).simple_serialize().unwrap(),
            MoveValue::Vector(utxo_ids.iter().map(|id| id.to_move_value()).collect())
                .simple_serialize()
                .unwrap(),
        ],
    }
}

pub fn balance_call(module_address: AccountAddress, faucet_object_id: ObjectID) -> FunctionCall {
    FunctionCall {
        function_id: FunctionId::new(
            ModuleId::new(module_address, MODULE_NAME.to_owned()),
            ident_str!("balance").to_owned(),
        ),
        ty_args: vec![],
        args: vec![faucet_object_id.to_move_value().simple_serialize().unwrap()],
    }
}

// The error code in Move
// const ErrorFaucetNotOpen: u64 = 1;
// const ErrorInvalidUTXO: u64 = 2;
// const ErrorFaucetNotEnoughRGas: u64 = 3;
// const ErrorAlreadyClaimed: u64 = 4;
// const ErrorUTXOValueIsZero: u64 = 5;

pub fn error_code_to_reason(error_code: u64) -> String {
    match error_code {
        1 => "Faucet is not open".to_string(),
        2 => "Invalid UTXO".to_string(),
        3 => "Faucet does not have enough RGas".to_string(),
        4 => "Already claimed".to_string(),
        5 => "UTXO value is zero".to_string(),
        _ => "Unknown error".to_string(),
    }
}
