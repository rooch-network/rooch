// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, language_storage::ModuleId,
};
use moveos_types::{
    move_std::string::MoveString, move_types::FunctionId, state::MoveState,
    transaction::FunctionCall,
};

pub const MODULE_NAME: &IdentStr = ident_str!("tweet_fetcher");

pub const FETCH_TWEET_FUNCTION: &IdentStr = ident_str!("fetch_tweet_entry");

pub fn fetch_tweet_function_call(module_address: AccountAddress, tweet_id: String) -> FunctionCall {
    FunctionCall {
        function_id: FunctionId::new(
            ModuleId::new(module_address, MODULE_NAME.to_owned()),
            FETCH_TWEET_FUNCTION.to_owned(),
        ),
        ty_args: vec![],
        args: vec![MoveString::from(tweet_id)
            .to_move_value()
            .simple_serialize()
            .unwrap()],
    }
}

// The error code in Move
// const ErrorInvalidRequestID: u64 = 1;
// const ErrorInvalidResponse: u64 = 2;
// const ErrorTooManyPendingRequests: u64 = 3;

pub fn error_code_to_reason(error_code: u64) -> String {
    match error_code {
        1 => "Invalid request ID".to_string(),
        2 => "Invalid response".to_string(),
        3 => "Too many pending requests".to_string(),
        _ => "Unknown error".to_string(),
    }
}
