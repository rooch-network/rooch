// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::RoochResult;

#[derive(Debug, Parser)]
pub struct SignTx {
    input_file: String,
    #[clap(long)]
    output_file: Option<String>,
}

#[async_trait]
impl CommandAction<String> for SignTx {
    async fn execute(self) -> RoochResult<String> {
        // Implement sign-tx logic here
        Ok(format!("Signed transaction from {}", self.input_file))
    }
}
