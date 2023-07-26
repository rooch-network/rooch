// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use codespan_reporting::diagnostic::Severity;
use move_cli::base::test;
use move_command_line_common::address::NumericalAddress;
use move_command_line_common::parser::NumberFormat;
use move_core_types::account_address::AccountAddress;
use move_package::BuildConfig;
use move_unit_test::extensions::set_extension_hook;
use move_vm_runtime::native_extensions::NativeContextExtensions;
use moveos_stdlib::natives::moveos_stdlib::raw_table::NativeTableContext;
use moveos_store::state_store::StateDBStore;
use moveos_store::MoveOSStore;
use moveos_verifier::build::build_model_with_test_attr;
use moveos_verifier::metadata::run_extended_checks;
use once_cell::sync::Lazy;
use rooch_framework::natives::{all_natives, GasParameters};
use std::{collections::BTreeMap, path::PathBuf};
use termcolor::Buffer;

#[derive(Parser)]
pub struct Test {
    #[clap(flatten)]
    pub test: test::Test,

    /// Named addresses for the move binary
    ///
    /// Example: alice=0x1234, bob=0x5678
    ///
    /// Note: This will fail if there are duplicates in the Move.toml file remove those first.
    #[clap(long, parse(try_from_str = crate::utils::parse_map), default_value = "")]
    pub(crate) named_addresses: BTreeMap<String, String>,
}

impl Test {
    pub fn execute(self, path: Option<PathBuf>, build_config: BuildConfig) -> anyhow::Result<()> {
        let mut build_config = build_config;
        build_config.additional_named_addresses = self
            .named_addresses
            .clone()
            .into_iter()
            .map(|(key, value)| (key, AccountAddress::from_hex_literal(&value).unwrap()))
            .collect();

        let root_path = path.clone().unwrap_or_else(|| PathBuf::from("."));

        build_config.dev_mode = true;

        let resolution_graph = build_config
            .clone()
            .resolution_graph_for_package(&root_path, &mut Vec::new())
            .expect("resolve package dep failed");

        let mut additional_named_address = BTreeMap::new();
        let _: Vec<_> = resolution_graph
            .extract_named_address_mapping()
            .map(|(name, addr)| {
                (additional_named_address.insert(
                    name.to_string(),
                    NumericalAddress::new(addr.into_bytes(), NumberFormat::Hex).into_inner(),
                ),)
            })
            .collect();

        let global_env = build_model_with_test_attr(&root_path, additional_named_address, None)?;

        let _ = run_extended_checks(&global_env);

        if global_env.diag_count(Severity::Warning) > 0 {
            let mut buffer = Buffer::ansi();
            global_env.report_diag(&mut buffer, Severity::Warning);
            let buffer_output = String::from_utf8_lossy(buffer.as_slice()).to_string();
            eprintln!("{}", buffer_output);
            if global_env.has_errors() {
                return Err(anyhow::Error::msg("extended checks failed"));
            }
        }

        //TODO define gas metering
        let cost_table = move_vm_test_utils::gas_schedule::INITIAL_COST_SCHEDULE.clone();
        let natives = all_natives(GasParameters::zeros());
        set_extension_hook(Box::new(new_moveos_natives_runtime));
        self.test
            .execute(path, build_config, natives, Some(cost_table))
    }
}

static STATEDBSTORE: Lazy<Box<StateDBStore>> =
    Lazy::new(|| Box::new(MoveOSStore::mock().unwrap().state_store));

fn new_moveos_natives_runtime(ext: &mut NativeContextExtensions) {
    let statedb_store = Lazy::force(&STATEDBSTORE).as_ref();
    let table_ext = NativeTableContext::new(statedb_store);

    ext.add(table_ext);
}
