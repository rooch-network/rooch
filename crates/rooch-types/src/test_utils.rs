// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::address::{RoochAddress, RoochSupportedAddress};
use crate::transaction::authenticator::Authenticator;
use crate::transaction::rooch::{RoochTransaction, RoochTransactionData};
use crate::transaction::{LedgerTransaction, TransactionSequenceInfo};
use ethers::types::H256;
use moveos_types::moveos_std::accumulator::AccumulatorInfo;
use rand::{thread_rng, Rng};

pub use moveos_types::test_utils::*;

pub fn random_rooch_transaction() -> RoochTransaction {
    let move_action_type = random_move_action_type();
    random_rooch_transaction_with_move_action(move_action_type)
}

pub fn random_ledger_transaction() -> LedgerTransaction {
    let rooch_transaction = random_rooch_transaction();

    let tx_order_signature = random_bytes();
    let accumulator_info = random_accumulator_info();
    let random_sequence_info = TransactionSequenceInfo::new(
        rand::random(),
        tx_order_signature,
        accumulator_info.accumulator_root,
        0,
        Some(accumulator_info),
    );
    LedgerTransaction::new_l2_tx(rooch_transaction, random_sequence_info)
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

pub fn random_accumulator_info() -> AccumulatorInfo {
    let mut rng = thread_rng();
    let num_leaves = rng.gen_range(1..=100) as u64;
    let num_nodes = rng.gen_range(1..=100) as u64;
    AccumulatorInfo::new(H256::random(), vec![], num_leaves, num_nodes)
}
