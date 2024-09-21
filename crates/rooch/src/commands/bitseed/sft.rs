// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::commands::bitseed::operation::{MintRecord, SplitRecord};
use anyhow::{ensure, Result};
use ciborium::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Content {
    pub content_type: String,
    pub body: Vec<u8>,
}

impl Content {
    pub fn new(content_type: String, body: Vec<u8>) -> Self {
        Self { content_type, body }
    }

    pub fn text(body: String) -> Self {
        Self::new("text/plain".to_string(), body.as_bytes().to_vec())
    }

    pub fn is_text(&self) -> bool {
        self.content_type == "text/plain"
    }

    pub fn as_text(&self) -> Result<String> {
        ensure!(self.is_text(), "Content is not text");
        Ok(String::from_utf8(self.body.clone())?)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SFT {
    pub tick: String,
    pub amount: u64,
    pub attributes: Option<Value>,
    pub content: Option<Content>,
}

impl SFT {
    pub fn new(
        tick: String,
        amount: u64,
        attributes: Option<Value>,
        content: Option<Content>,
    ) -> Self {
        Self {
            tick,
            amount,
            attributes,
            content,
        }
    }

    pub fn split(&mut self, amount: u64) -> Result<SFT> {
        if amount > self.amount {
            return Err(anyhow::anyhow!(
                "Split amount is greater than the SFT amount"
            ));
        }
        self.amount -= amount;
        Ok(SFT::new(
            self.tick.clone(),
            amount,
            self.attributes.clone(),
            self.content.clone(),
        ))
    }

    pub fn merge(&mut self, sft: SFT) -> Result<()> {
        if self.tick != sft.tick {
            return Err(anyhow::anyhow!("SFTs have different ticks"));
        }
        if self.attributes != sft.attributes {
            return Err(anyhow::anyhow!("SFTs have different attributes"));
        }
        if self.content != sft.content {
            return Err(anyhow::anyhow!("SFTs have different content"));
        }
        self.amount += sft.amount;
        Ok(())
    }

    pub fn to_mint_record(&self) -> MintRecord {
        MintRecord { sft: self.clone() }
    }

    pub fn to_split_record(&self) -> SplitRecord {
        SplitRecord { sft: self.clone() }
    }
}
