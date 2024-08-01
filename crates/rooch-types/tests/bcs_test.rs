// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId, u256::U256,
};
use moveos_types::{
    move_types::FunctionId,
    moveos_std::{object::ObjectMeta, tx_context::TxContext},
    transaction::{FunctionCall, VerifiedMoveAction, VerifiedMoveOSTransaction},
};
use serde::de::Visitor;
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize)]
pub struct NewVerifiedMoveOSTransaction {
    pub root: ObjectMeta,
    pub ctx: TxContext,
    pub action: VerifiedMoveAction,
}

impl<'de> Deserialize<'de> for NewVerifiedMoveOSTransaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NewVerifiedMoveOSTransactionVisitor;

        impl<'de> Visitor<'de> for NewVerifiedMoveOSTransactionVisitor {
            type Value = NewVerifiedMoveOSTransaction;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct NewVerifiedMoveOSTransaction")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                //let object_id = seq.next_element::<ObjectID>()?.unwrap();
                let root = seq.next_element::<ObjectMeta>()?.unwrap();
                let ctx = seq.next_element::<TxContext>()?.unwrap();
                let action = seq.next_element::<VerifiedMoveAction>()?.unwrap();
                let _pre_execution_functions: Vec<FunctionCall> =
                    match seq.next_element::<Vec<FunctionCall>>() {
                        Ok(Some(pre_execution_functions)) => pre_execution_functions,
                        Ok(None) => vec![],
                        Err(e) => {
                            println!("error: {:?}", e);
                            vec![]
                        }
                    };
                let _post_execution_functions: Vec<FunctionCall> =
                    match seq.next_element::<Vec<FunctionCall>>() {
                        Ok(Some(post_execution_functions)) => post_execution_functions,
                        Ok(None) => vec![],
                        Err(e) => {
                            println!("error: {:?}", e);
                            vec![]
                        }
                    };
                Ok(NewVerifiedMoveOSTransaction { root, ctx, action })
            }
        }

        deserializer.deserialize_struct(
            "NewVerifiedMoveOSTransaction",
            &[
                "root",
                "ctx",
                "action",
                "pre_execution_functions",
                "post_execution_functions",
            ],
            NewVerifiedMoveOSTransactionVisitor,
        )
    }
}

#[test]
fn test() {
    let root = ObjectMeta::genesis_root();
    let ctx = TxContext::random_for_testing_only();
    let action = VerifiedMoveAction::Function {
        call: FunctionCall::new(
            FunctionId::new(
                ModuleId::new(AccountAddress::ONE, Identifier::from_str("test").unwrap()),
                Identifier::from_str("test").unwrap(),
            ),
            vec![],
            vec![],
        ),
        bypass_visibility: false,
    };
    let old = VerifiedMoveOSTransaction::new(root, ctx, action);
    let old_bytes = bcs::to_bytes(&old).unwrap();
    let new: NewVerifiedMoveOSTransaction = bcs::from_bytes(&old_bytes).unwrap();
    let new_bytes = bcs::to_bytes(&new).unwrap();
    let _new2: NewVerifiedMoveOSTransaction = bcs::from_bytes(&new_bytes).unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OldStruct {
    value: u64,
}

#[derive(Debug, Clone, Serialize)]
struct NewStruct {
    value: u64,
    #[serde(default)]
    hash: U256,
}

impl<'de> Deserialize<'de> for NewStruct {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct NewStructVisitor;

        impl<'de> serde::de::Visitor<'de> for NewStructVisitor {
            type Value = NewStruct;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct NewStruct")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                println!("visit_seq");
                println!("seq_size: {:?}", seq.size_hint());
                let value = match seq.next_element() {
                    Ok(Some(value)) => value,
                    Ok(None) => 0,
                    Err(e) => {
                        println!("error: {:?}", e);
                        0
                    }
                };
                println!("value: {:?}", value);
                let hash = match seq.next_element() {
                    Ok(Some(hash)) => hash,
                    Ok(None) => U256::default(),
                    Err(e) => {
                        println!("error: {:?}", e);
                        U256::default()
                    }
                };
                println!("hash: {:?}", hash);
                Ok(NewStruct { value, hash })
            }
        }

        deserializer.deserialize_struct("NewStruct", &["value", "hash"], NewStructVisitor)
    }
}

#[test]
fn test2() {
    let old = OldStruct { value: 1 };
    let old_bytes = bcs::to_bytes(&old).unwrap();
    let mut new: NewStruct = bcs::from_bytes(&old_bytes).unwrap();
    new.hash = U256::from(2u64);
    let new_bytes = bcs::to_bytes(&new).unwrap();
    let _new2: NewStruct = bcs::from_bytes(&new_bytes).unwrap();
}
