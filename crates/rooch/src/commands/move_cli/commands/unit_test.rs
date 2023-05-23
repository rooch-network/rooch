// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_cli::base::test;
use move_package::BuildConfig;
use move_unit_test::extensions::set_extension_hook;
use move_vm_runtime::native_extensions::NativeContextExtensions;
use moveos_statedb::StateDB;
use moveos_stdlib::natives::moveos_stdlib::raw_table::NativeTableContext;
use moveos_stdlib::natives::{all_natives, GasParameters};
use once_cell::sync::Lazy;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Test {
    #[clap(flatten)]
    pub test: test::Test,
}

impl Test {
    pub fn execute(self, path: Option<PathBuf>, build_config: BuildConfig) -> anyhow::Result<()> {
        //TODO define gas metering
        let cost_table = move_vm_test_utils::gas_schedule::INITIAL_COST_SCHEDULE.clone();
        let natives = all_natives(GasParameters::zeros());

        set_extension_hook(Box::new(new_moveos_natives_runtime));
        self.test
            .execute(path, build_config, natives, Some(cost_table))
    }
}

static STATEDB: Lazy<Box<StateDB>> = Lazy::new(|| Box::new(StateDB::new_with_memory_store()));

fn new_moveos_natives_runtime(ext: &mut NativeContextExtensions) {
    let statedb = Lazy::force(&STATEDB).as_ref();
    let table_ext = NativeTableContext::new(statedb);

    ext.add(table_ext);
}
