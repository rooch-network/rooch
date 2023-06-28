// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_integration_test_runner::run_test;

datatest_stable::harness!(run_test, "tests", r".*\.(mvir|move)$");
