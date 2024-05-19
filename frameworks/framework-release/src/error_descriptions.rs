// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::STATIC_FRAMEWORK_DIR;
use framework_types::addresses::*;
use move_core_types::{account_address::AccountAddress, errmap::ErrorMapping};
use once_cell::sync::Lazy;
use std::collections::BTreeMap;

pub static ERROR_DESCRIPTIONS: Lazy<BTreeMap<AccountAddress, ErrorMapping>> = Lazy::new(|| {
    let mut error_descriptions = BTreeMap::new();

    let move_stdlib_err: ErrorMapping = bcs::from_bytes(
        STATIC_FRAMEWORK_DIR
            .get_file("latest/move_stdlib_error_description.errmap")
            .unwrap()
            .contents(),
    )
    .unwrap();
    error_descriptions.insert(MOVE_STD_ADDRESS, move_stdlib_err);

    let moveos_std_err: ErrorMapping = bcs::from_bytes(
        STATIC_FRAMEWORK_DIR
            .get_file("latest/moveos_stdlib_error_description.errmap")
            .unwrap()
            .contents(),
    )
    .unwrap();
    error_descriptions.insert(MOVEOS_STD_ADDRESS, moveos_std_err);

    let rooch_framework_err: ErrorMapping = bcs::from_bytes(
        STATIC_FRAMEWORK_DIR
            .get_file("latest/rooch_framework_error_description.errmap")
            .unwrap()
            .contents(),
    )
    .unwrap();

    error_descriptions.insert(ROOCH_FRAMEWORK_ADDRESS, rooch_framework_err);

    let bitcoin_move_err: ErrorMapping = bcs::from_bytes(
        STATIC_FRAMEWORK_DIR
            .get_file("latest/bitcoin_move_error_description.errmap")
            .unwrap()
            .contents(),
    )
    .unwrap();

    error_descriptions.insert(BITCOIN_MOVE_ADDRESS, bitcoin_move_err);

    let rooch_nursery_err: ErrorMapping = bcs::from_bytes(
        STATIC_FRAMEWORK_DIR
            .get_file("latest/rooch_nursery_error_description.errmap")
            .unwrap()
            .contents(),
    )
    .unwrap();

    error_descriptions.insert(ROOCH_NURSERY_ADDRESS, rooch_nursery_err);

    error_descriptions
});

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_error_descriptions() {
        let error_descriptions = ERROR_DESCRIPTIONS.clone();
        let error_mapping = error_descriptions.get(&MOVEOS_STD_ADDRESS).unwrap();
        //println!("{:?}",error_mapping.module_error_maps);
        let description = error_mapping.get_explanation("0x2::object", 1);
        //println!("{:?}",description);
        assert!(description.is_some());
        let description = description.unwrap();
        assert_eq!(description.code_name.as_str(), "ErrorAlreadyExists");
    }
}
