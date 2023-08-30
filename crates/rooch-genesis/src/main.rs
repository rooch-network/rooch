// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use rooch_genesis::crate_dir;

fn main() {
    //if the current directory is not the crate directory
    //like we run from the root directory `cargo run -p rooch-genesis`, the generated stdlib document's link will be broken
    //So, we need to set the current directory to the crate directory
    std::env::set_current_dir(crate_dir()).unwrap();
    rooch_genesis_builder::build_and_save_stdlib().unwrap();
}
