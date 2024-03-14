// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct DataConfig {
    pub ord_cli_path: String,
    pub ord_data_path: String,
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            ord_cli_path: "".to_string(),
            // ord_cli_path: "/Users/Baichuan/ordinals/ord/target/release/";
            ord_data_path: "/Users/Baichuan/Downloads/ord_data/".to_string(),
        }
    }
}
