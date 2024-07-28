// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportInfoView {
    pub mnemonic_phrase: Option<String>,
    pub encoded_private_key: Option<String>,
}

impl ExportInfoView {
    pub fn new_mnemonic_phrase(mnemonic_phrase: String) -> Self {
        Self {
            mnemonic_phrase: Some(mnemonic_phrase),
            encoded_private_key: None,
        }
    }

    pub fn new_encoded_private_key(encoded_private_key: String) -> Self {
        Self {
            mnemonic_phrase: None,
            encoded_private_key: Some(encoded_private_key),
        }
    }
}
