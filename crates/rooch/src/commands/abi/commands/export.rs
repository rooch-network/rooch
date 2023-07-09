// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use clap::Parser;
use std::fmt::Debug;
use std::str::FromStr;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use moveos_types::move_types::{FunctionId};
use moveos_types::transaction::{ MoveAction, ScriptCall, FunctionCall};
use rooch_types::{error::RoochResult};

#[derive(Debug, Parser)]
pub struct ExportCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<()> for ExportCommand {
    async fn execute(self) -> RoochResult<()> {
        println!("export command execute");
        export_typescript()
    }
}

use serde::{Deserialize, Serialize};
use serde_reflection::{Tracer, TracerConfig, Samples};

#[derive(Serialize, Deserialize)]
struct Test {
    a: Vec<u64>,
    b: (u32, u32),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub enum TypeTag {
    // alias for compatibility with old json serialized data.
    #[serde(rename = "bool", alias = "Bool")]
    Bool,
    #[serde(rename = "u8", alias = "U8")]
    U8,
    #[serde(rename = "u64", alias = "U64")]
    U64,
    #[serde(rename = "u128", alias = "U128")]
    U128,
    #[serde(rename = "address", alias = "Address")]
    Address,
    #[serde(rename = "signer", alias = "Signer")]
    Signer,
    #[serde(rename = "vector", alias = "Vector")]
    Vector(Box<TypeTag>),
    #[serde(rename = "struct", alias = "Struct")]
    Struct(Box<StructTag>),

    // NOTE: Added in bytecode version v6, do not reorder!
    #[serde(rename = "u16", alias = "U16")]
    U16,
    #[serde(rename = "u32", alias = "U32")]
    U32,
    #[serde(rename = "u256", alias = "U256")]
    U256,
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct StructTag {
    pub address: AccountAddress,
    pub module: Identifier,
    pub name: Identifier,
    // alias for compatibility with old json serialized data.
    #[serde(rename = "type_args", alias = "type_params")]
    pub type_params: TypeTag,
}
 
 
fn export_typescript() -> RoochResult<()>{
    // Obtain the Serde format of `Test`. (In practice, formats are more likely read from a file.)
    let mut tracer = Tracer::new(TracerConfig::default());
    // Create a store to hold samples of Rust values.
    let mut samples = Samples::new();

    let account_address = AccountAddress::random();
    tracer.trace_value(&mut samples, &account_address).unwrap();

    let id = Identifier::from_str("account").unwrap();
    tracer.trace_value(&mut samples, &id).unwrap();

    let tag = StructTag{
        address: AccountAddress::from_str("0x1").unwrap(),
        module: Identifier::from_str("account").unwrap(),
        name: Identifier::from_str("init").unwrap(),
        type_params: TypeTag::U16,
    };
    
    tracer.trace_value(&mut samples, &tag).unwrap();
    tracer.trace_type::<StructTag>(&mut samples).unwrap();

    tracer.trace_simple_type::<MoveAction>().unwrap();
    tracer.trace_simple_type::<ScriptCall>().unwrap();
    tracer.trace_simple_type::<FunctionCall>().unwrap();
    tracer.trace_simple_type::<FunctionId>().unwrap();
    tracer.trace_simple_type::<TypeTag>().unwrap();

    let registry = tracer.registry().unwrap();

    let data: String = serde_yaml::to_string(&registry).unwrap();
    println!("export rooch_types.yaml: {data}");

    Ok(())
}
