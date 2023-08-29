// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_genesis::{crate_dir, GenesisPackage};

fn main() {
    //if the current directory is not the crate directory
    //like we run from the root directory `cargo run -p rooch-genesis`, the generated stdlib document's link will be broken
    //So, we need to set the current directory to the crate directory
    std::env::set_current_dir(crate_dir()).unwrap();
    GenesisPackage::build_stdlib()
        .unwrap()
        .save_to_file(GenesisPackage::stdlib_file())
        .unwrap();
}
