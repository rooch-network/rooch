// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, language_storage::ModuleId,
};
use moveos_types::{
    move_std::string::MoveString, move_types::FunctionId, state::MoveState,
    transaction::FunctionCall,
};

pub const MODULE_NAME: &IdentStr = ident_str!("twitter_account");

pub const VERIFY_AND_BINDING_TWITTER_ACCOUNT_FUNCTION: &IdentStr =
    ident_str!("verify_and_binding_twitter_account");
pub const CHECK_BINDING_TWEET_FUNCTION_CALL: &IdentStr = ident_str!("check_binding_tweet");

pub fn verify_and_binding_twitter_account_function_call(
    module_address: AccountAddress,
    tweet_id: String,
) -> FunctionCall {
    FunctionCall {
        function_id: FunctionId::new(
            ModuleId::new(module_address, MODULE_NAME.to_owned()),
            VERIFY_AND_BINDING_TWITTER_ACCOUNT_FUNCTION.to_owned(),
        ),
        ty_args: vec![],
        args: vec![MoveString::from(tweet_id)
            .to_move_value()
            .simple_serialize()
            .unwrap()],
    }
}

pub fn check_binding_tweet_function_call(
    module_address: AccountAddress,
    tweet_id: String,
) -> FunctionCall {
    FunctionCall {
        function_id: FunctionId::new(
            ModuleId::new(module_address, MODULE_NAME.to_owned()),
            CHECK_BINDING_TWEET_FUNCTION_CALL.to_owned(),
        ),
        ty_args: vec![],
        args: vec![MoveString::from(tweet_id)
            .to_move_value()
            .simple_serialize()
            .unwrap()],
    }
}

// The error code in Move
// const ErrorTweetNotFound: u64 = 1;
// const ErrorAccountAlreadyBound: u64 = 2;
// const ErrorAuthorAddressNotFound: u64 = 3;
// const ErrorTweetBindingMessageInvalidPrefix: u64 = 4;
// const ErrorTweetBindingMessageMissingHashtag: u64 = 5;
// const ErrorTweetBindingMessageInvalidAddress: u64 = 6;

pub fn error_code_to_reason(error_code: u64) -> String {
    match error_code {
        1 => "Tweet Not Found, please wait the Oracles to finish fetch data.".to_string(),
        2 => "The Twitter account already bound".to_string(),
        3 => "Can not find Bitcoin address in the tweet".to_string(),
        4 => "The binding message's prefix is invalid.".to_string(),
        5 => "The binding message missing required Hash Tag".to_string(),
        6 => "The binding message has invalid Bitcoin address".to_string(),
        _ => "Unknown error".to_string(),
    }
}
