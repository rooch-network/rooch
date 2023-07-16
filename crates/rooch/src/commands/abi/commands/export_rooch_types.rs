// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use clap::Parser;
use std::fmt::Debug;
use std::fs;
use std::path::Path;
use serde_reflection::{Tracer, TracerConfig, Samples};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{TypeTag, StructTag},
};
use moveos_types::transaction::{ MoveAction};
use rooch_types::{error::RoochResult};
use crate::cli_types::{CommandAction, WalletContextOptions};

#[derive(Debug, Parser)]
pub struct ExportRoochTypesCommand {
    #[clap(flatten)]
    pub context_options: WalletContextOptions,

    #[clap(long, default_value = "./target/rooch_types.yaml")]
    pub file_path: String,
}

#[async_trait]
impl CommandAction<()> for ExportRoochTypesCommand {
    async fn execute(self) -> RoochResult<()> {
        export_rooch_types_yaml(&self.file_path)
    }
}

fn export_rooch_types_yaml(file_path: &String) -> RoochResult<()>{
    let mut tracer = Tracer::new(TracerConfig::default());

    // Predefine StructTag to prevent recursive definitions from reporting errors.
    let mut samples = Samples::new();
    let example_struct_tag = StructTag {
        address: AccountAddress::random(), 
        module: Identifier::new("Module").unwrap(),
        name: Identifier::new("Name").unwrap(),
        type_params: vec!(TypeTag::Bool),
    };
    tracer.trace_value(&mut samples, &example_struct_tag).unwrap();

    let example_type_tag = TypeTag::Struct(Box::new(example_struct_tag));
    tracer.trace_value(&mut samples, &example_type_tag).unwrap();

    // Define TypeTag and MoveAction
    tracer.trace_type::<TypeTag>(&mut samples).unwrap();
    tracer.trace_type::<MoveAction>(&mut samples).unwrap();

    match tracer.registry() {
        Ok(registry)=>{
            let data: String = serde_json::to_string_pretty(&registry).unwrap();

            // Since serde_yaml does not support nested enumerations, the registry is first converted to json and then converted to yaml.
            let json_value: serde_json::Value = serde_json::from_str(data.as_str()).unwrap();
            let yaml_string = serde_yaml::to_string(&json_value).unwrap();

            let path = Path::new(file_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?; // 创建所有父目录
            }

            fs::write(path, yaml_string)?; // 创建文件并写入数据

            println!("export rooch types to file: {file_path} ok!");
        },
        Err(e)=>{
            let msg = e.explanation();
            println!("export rooch_types.yaml error: {msg}");
        }
    }

    Ok(())
}
