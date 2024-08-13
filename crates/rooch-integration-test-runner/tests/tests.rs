// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_integration_test_runner::run_test;
use std::path::Path;
use tokio::runtime::Runtime;

pub fn async_run_test(path: &Path) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let runtime =
        Runtime::new().expect("Failed to create Tokio runtime when execute async run test ");
    runtime.block_on(async { run_test(path) })
}

datatest_stable::harness!(run_test, "tests", r".*\.(mvir|move)$");
