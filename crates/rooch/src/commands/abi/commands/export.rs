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

    tracer.trace_simple_type::<FunctionId>().unwrap();
    //
    //tracer.trace_simple_type::<StructTag>().unwrap();

    // Create a store to hold samples of Rust values.

    let mut samples = Samples::new();
    let example_struct_tag = StructTag {
        address: AccountAddress::random(), 
        module: Identifier::new("Module").unwrap(),
        name: Identifier::new("Name").unwrap(),
        type_params: TypeTag::Bool
    };

    match tracer.trace_value(&mut samples, &example_struct_tag) {
        Ok(_) => (),  
        Err(e) => {
            println!("Error occurred: {}", e.explanation());
            println!("{:#?}", e);
            std::process::exit(1);
        }
    }

    let example_type_tag = TypeTag::Struct(Box::new(example_struct_tag));
    tracer.trace_value(&mut samples, &example_type_tag).unwrap();

    tracer.trace_type::<TypeTag>(&mut samples).unwrap();
    tracer.trace_type::<MoveAction>(&mut samples).unwrap();

    match tracer.registry() {
        Ok(registry)=>{
            let data: String = serde_json::to_string_pretty(&registry).unwrap();
            println!("export rooch_types.yaml: {data}");
        },
        Err(e)=>{
            let msg = e.explanation();
            println!("export rooch_types.yaml error: {msg}");
        }
    }

    Ok(())
}
