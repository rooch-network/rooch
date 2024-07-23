// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use crate::cli_types::WalletContextOptions;
use crate::commands::move_cli::print_serialized_success;
use async_trait::async_trait;
use bcs;
use clap::Parser;
use move_cli::{base::reroot_path, Move};
use moveos_verifier::build::run_verifier;
use rooch_types::error::RoochResult;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};

/// Build the package at `path`. If no path is provided defaults to current directory.
#[derive(Parser)]
#[clap(name = "build")]
pub struct BuildCommand {
    /// Named addresses for the move binary
    ///
    /// Example: alice=0x1234, bob=default, alice2=alice
    ///
    /// Note: This will fail if there are duplicates in the Move.toml file remove those first.
    #[clap(long, value_parser = crate::utils::parse_map::<String, String>, default_value = "")]
    pub(crate) named_addresses: BTreeMap<String, String>,

    #[clap(flatten)]
    config_options: WalletContextOptions,

    #[clap(flatten)]
    move_args: Move,

    /// If true, export package binary to the output directory with name `package.blob`
    #[clap(long, default_value = "false")]
    export: bool,

    /// Return command outputs in json format
    #[clap(long, default_value = "false")]
    json: bool,
}

#[async_trait]
impl CommandAction<Option<Value>> for BuildCommand {
    async fn execute(self) -> RoochResult<Option<Value>> {
        let path = self.move_args.package_path;
        let config = self.move_args.build_config;

        let context = self.config_options.build()?;

        let mut config = config;
        config
            .additional_named_addresses
            .extend(context.parse_and_resolve_addresses(self.named_addresses)?);

        let rerooted_path = reroot_path(path)?;
        if config.fetch_deps_only {
            if config.test_mode {
                config.dev_mode = true;
            }
            config.download_deps_for_package(&rerooted_path, &mut std::io::stdout())?;
            return print_serialized_success(self.json);
        }

        let config_cloned = config.clone();

        let mut package = config.compile_package_no_exit(&rerooted_path, &mut std::io::stdout())?;

        run_verifier(rerooted_path.clone(), config_cloned.clone(), &mut package)?;

        if self.export {
            let export_path = match &config_cloned.install_dir {
                None => rerooted_path
                    .join("build")
                    .join(package.compiled_package_info.package_name.as_str())
                    .join("package.blob"),
                Some(value) => value.clone().join("package.blob"),
            };

            let blob = package
                .root_compiled_units
                .iter()
                .map(|unit| unit.unit.serialize(None))
                .collect::<Vec<_>>();

            let mut file = BufWriter::new(File::create(export_path.clone())?);
            bcs::serialize_into(&mut file, &blob)?;
            file.flush()?;

            println!("Exported package to {}", export_path.display());
        }

        print_serialized_success(self.json)
    }
}
