// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_genesis::GenesisPackage;

fn main() {
    GenesisPackage::build_stdlib()
        .unwrap()
        .save_to_file(GenesisPackage::stdlib_file())
        .unwrap();
}
