// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::cli_types::CommandAction;
use async_trait::async_trait;
use clap::Parser;
use rooch_types::error::{RoochError, RoochResult};

/// Tool for reverse hex encoding
#[derive(Debug, Parser)]
pub struct HexCommand {
    /// Hex string to decode
    /// The command will reverse the bytes of the hex string, and then encode it back to hex
    /// Example: `0x1234` will be reversed to `3412`
    /// This is useful for encoding and decoding little-endian data, such as Bitcoin TxId or BlockHash
    hex: String,
}

#[async_trait]
impl CommandAction<String> for HexCommand {
    async fn execute(self) -> RoochResult<String> {
        let hex = self.hex.strip_prefix("0x").unwrap_or(&self.hex);
        let mut bytes =
            hex::decode(hex).map_err(|e| RoochError::CommandArgumentError(e.to_string()))?;
        bytes.reverse();
        Ok(hex::encode(bytes))
    }
}
