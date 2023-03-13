// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    transaction_argument::TransactionArgument,
};
use serde::{Deserialize, Serialize};
use statedb::HashValue;

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Script {
    #[serde(with = "serde_bytes")]
    pub code: Vec<u8>,
    pub ty_args: Vec<TypeTag>,
    pub args: Vec<TransactionArgument>,
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub module: ModuleId,
    pub function: Identifier,
    pub ty_args: Vec<TypeTag>,
    pub args: Vec<Vec<u8>>,
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MoveTransaction {
    //Execute a Move script
    Script(Script),
    //Execute a Move function
    Function(Function),
    //Publish Move modules
    ModuleBundle(Vec<Vec<u8>>),
}

pub trait AbstractTransaction {
    fn senders(&self) -> Vec<AccountAddress>;
    fn into_move_transaction(self) -> MoveTransaction;
    fn txn_hash(&self) -> HashValue;
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimpleTransaction {
    pub sender: AccountAddress,
    pub txn: MoveTransaction,
}

impl SimpleTransaction {
    pub fn new(sender: AccountAddress, txn: MoveTransaction) -> Self {
        Self { sender, txn }
    }
}

impl AbstractTransaction for SimpleTransaction {
    fn senders(&self) -> Vec<AccountAddress> {
        vec![self.sender]
    }

    fn into_move_transaction(self) -> MoveTransaction {
        self.txn
    }

    fn txn_hash(&self) -> HashValue {
        HashValue::sha3_256_of(bcs::to_bytes(&self).unwrap().as_slice())
    }
}
