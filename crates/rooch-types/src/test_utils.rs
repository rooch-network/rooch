// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::{RoochAddress, RoochSupportedAddress};
use crate::transaction::authenticator::Authenticator;
use crate::transaction::ethereum::EthereumTransaction;
use crate::transaction::rooch::{RoochTransaction, RoochTransactionData};
use crate::transaction::TypedTransaction;
use ethers::types::{Bytes, U256};
use rand::{thread_rng, Rng};

pub use moveos_types::test_utils::*;

pub fn random_typed_transaction() -> TypedTransaction {
    let mut rng = thread_rng();
    let n = rng.gen_range(1..=100);
    if n % 2 == 0 {
        TypedTransaction::Rooch(random_rooch_transaction())
    } else {
        TypedTransaction::Ethereum(random_ethereum_transaction())
    }
}

/// Returns rooch typed transaction which move action is move function
pub fn random_typed_transaction_for_rooch_function() -> TypedTransaction {
    TypedTransaction::Rooch(random_rooch_transaction_with_move_action(
        MoveActionType::Function,
    ))
}

pub fn random_rooch_transaction() -> RoochTransaction {
    let move_action_type = random_move_action_type();
    random_rooch_transaction_with_move_action(move_action_type)
}

pub fn random_rooch_transaction_with_move_action(move_action: MoveActionType) -> RoochTransaction {
    let mut rng = thread_rng();
    let sequence_number = rng.gen_range(1..=100);
    let tx_data = RoochTransactionData::new_for_test(
        RoochAddress::random(),
        sequence_number,
        random_move_action_with_action_type(move_action.action_type()),
    );

    let mut rng = thread_rng();
    let auth_validator_id = rng.gen_range(1..=100);
    let authenticator = Authenticator::new(auth_validator_id, random_bytes());

    RoochTransaction::new(tx_data, authenticator)
}

pub fn random_ethereum_transaction() -> EthereumTransaction {
    let sender = RoochAddress::random();
    let sequence_number = U256::zero();
    let move_action_type = random_move_action_type();
    let action = random_move_action_with_action_type(move_action_type.action_type());
    let action_bytes =
        Bytes::try_from(bcs::to_bytes(&action).unwrap()).expect("Convert action to bytes failed.");
    EthereumTransaction::new_for_test(sender, sequence_number, action_bytes)
}
