// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, language_storage::StructTag,
};
use moveos_types::{
    move_std::string::MoveString,
    moveos_std::object::{self, ObjectID},
};

pub const MODULE_NAME: &IdentStr = ident_str!("tweet_v2");

pub const STRUCT_NAME: &IdentStr = ident_str!("Tweet");

pub fn tweet_struct_tag(module_addresss: AccountAddress) -> StructTag {
    StructTag {
        address: module_addresss,
        module: MODULE_NAME.to_owned(),
        name: STRUCT_NAME.to_owned(),
        type_params: vec![],
    }
}

pub fn tweet_object_id(module_addresss: AccountAddress, tweet_id: String) -> ObjectID {
    object::custom_object_id(
        &MoveString::from(tweet_id),
        &tweet_struct_tag(module_addresss),
    )
}

// The error code in Move
//const ErrorTweetNotFound: u64 = 1;
//const ErrorTweetOwnerNotMatch: u64 = 2;
//const ErrorInvalidTweetJson: u64 = 3;

pub fn error_code_to_reason(error_code: u64) -> String {
    match error_code {
        1 => "Tweet not found".to_string(),
        2 => "Tweet owner not match".to_string(),
        3 => "Invalid tweet json".to_string(),
        _ => "Unknown error".to_string(),
    }
}
