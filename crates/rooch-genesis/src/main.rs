// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_genesis::{BuildOption, RoochGenesis};

fn main() {
    let genesis = RoochGenesis::build_with_option(BuildOption::Fresh).unwrap();
    genesis.genesis_package.save().unwrap();
}
