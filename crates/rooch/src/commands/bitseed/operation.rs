// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{
    inscription::{BitseedInscription, InscriptionBuilder},
    sft::SFT,
};
use anyhow::{anyhow, bail, Result};
use ciborium::Value;
use rooch_types::bitcoin::ord::{Inscription, InscriptionRecord};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Value as JSONValue;
use std::io::Cursor;

pub trait AsSFT {
    fn as_sft(&self) -> SFT;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeployRecord {
    pub tick: String,
    // The total supply of the Inscription
    pub amount: u64,
    pub generator: Option<String>,
    pub factory: Option<String>,
    pub repeat: u64,
    pub deploy_args: Vec<u8>,
}

impl DeployRecord {
    pub fn new_deploy_record(
        tick: String,
        amount: u64,
        generator: Option<String>,
        factory: Option<String>,
        repeat: u64,
        deploy_args: Vec<u8>,
    ) -> Self {
        Self {
            tick,
            amount,
            generator,
            factory,
            repeat,
            deploy_args,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MintRecord {
    pub sft: SFT,
}

impl AsSFT for MintRecord {
    fn as_sft(&self) -> SFT {
        self.sft.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplitRecord {
    pub sft: SFT,
}

impl AsSFT for SplitRecord {
    fn as_sft(&self) -> SFT {
        self.sft.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MergeRecord {
    pub sft: SFT,
}

impl AsSFT for MergeRecord {
    fn as_sft(&self) -> SFT {
        self.sft.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    Deploy(DeployRecord),
    Mint(MintRecord),
    Split(SplitRecord),
    Merge(MergeRecord),
}

impl Operation {
    pub fn to_inscription(self) -> InscriptionRecord {
        let op = self.op();
        match self {
            Operation::Deploy(record) => {
                let cursor = Cursor::new(record.deploy_args);
                let cbor_value: Value = ciborium::from_reader(cursor).unwrap();

                let mut att_values = vec![];
                if let Some(generator) = record.generator {
                    att_values.push((Value::Text("generator".to_string()), Value::Text(generator)));
                }

                if let Some(factory) = record.factory {
                    att_values.push((Value::Text("factory".to_string()), Value::Text(factory)));
                }

                att_values.push((
                    Value::Text("repeat".to_string()),
                    Value::Integer(record.repeat.into()),
                ));

                att_values.push((Value::Text("deploy_args".to_string()), cbor_value));

                let attributes = ciborium::Value::Map(att_values);
                InscriptionBuilder::new()
                    .op(op.clone())
                    .tick(record.tick.clone())
                    .amount(record.amount)
                    .attributes(attributes)
                    .finish()
            }
            Operation::Mint(record) => {
                let mut builder = InscriptionBuilder::new()
                    .op(op.clone())
                    .tick(record.sft.tick.clone())
                    .amount(record.sft.amount);
                if let Some(attributes) = record.sft.attributes {
                    builder = builder.attributes(attributes);
                }
                if let Some(content) = record.sft.content {
                    builder = builder.content(content)
                }
                builder.finish()
            }
            Operation::Split(record) => {
                let mut builder = InscriptionBuilder::new()
                    .op(op.clone())
                    .tick(record.sft.tick.clone())
                    .amount(record.sft.amount);
                if let Some(attributes) = record.sft.attributes {
                    builder = builder.attributes(attributes);
                }
                if let Some(content) = record.sft.content {
                    builder = builder.content(content)
                }
                builder.finish()
            }
            Operation::Merge(record) => {
                let mut builder = InscriptionBuilder::new()
                    .op(op.clone())
                    .tick(record.sft.tick.clone())
                    .amount(record.sft.amount);
                if let Some(attributes) = record.sft.attributes {
                    builder = builder.attributes(attributes);
                }
                if let Some(content) = record.sft.content {
                    builder = builder.content(content)
                }
                builder.finish()
            }
        }
    }

    pub fn from_inscription(inscription: Inscription) -> Result<Self> {
        let bitseed_inscription = BitseedInscription::new(inscription)?;
        let op = bitseed_inscription.op()?;
        let tick = bitseed_inscription.tick()?;
        let amount = bitseed_inscription.amount()?;
        let content = bitseed_inscription.content();

        match op.as_ref() {
            "deploy" => {
                let generator = bitseed_inscription
                    .get_attribute("generator")
                    .and_then(|v| v.as_text().map(|txt| txt.to_owned()));
                let factory = bitseed_inscription
                    .get_attribute("factory")
                    .and_then(|v| v.as_text().map(|txt| txt.to_owned()));
                match (&generator, &factory) {
                    (Some(_), Some(_)) => bail!("generator and factory are mutually exclusive"),
                    (None, None) => bail!("missing generator or factory"),
                    _ => {}
                }
                let repeat = bitseed_inscription
                    .get_attribute("repeat")
                    .ok_or_else(|| anyhow!("missing repeat"))?
                    .as_integer()
                    .ok_or_else(|| anyhow!("repeat is not an integer"))?
                    .try_into()?;
                let deploy_args_value = bitseed_inscription
                    .get_attribute("deploy_args")
                    .ok_or_else(|| anyhow!("missing deploy_args"))?;

                let mut deploy_args = Vec::new();
                ciborium::into_writer(&deploy_args_value, &mut deploy_args)
                    .expect("ciborium marshal failed");

                Ok(Operation::Deploy(DeployRecord::new_deploy_record(
                    tick,
                    amount,
                    generator,
                    factory,
                    repeat,
                    deploy_args,
                )))
            }
            "mint" | "split" | "merge" => {
                let attributes = bitseed_inscription.attributes();
                let sft = SFT {
                    tick,
                    amount,
                    attributes,
                    content,
                };

                let op = match op.as_ref() {
                    "mint" => Operation::Mint(MintRecord { sft }),
                    "split" => Operation::Split(SplitRecord { sft }),
                    "merge" => Operation::Merge(MergeRecord { sft }),
                    _ => unreachable!(), // We already know it's one of the three.
                };

                Ok(op)
            }
            _ => {
                bail!("unknown op: {}", op)
            }
        }
    }

    pub fn is_deploy(&self) -> bool {
        matches!(self, Operation::Deploy(_))
    }

    pub fn as_deploy(&self) -> Option<&DeployRecord> {
        match self {
            Operation::Deploy(record) => Some(record),
            _ => None,
        }
    }

    pub fn is_mint(&self) -> bool {
        matches!(self, Operation::Mint(_))
    }

    pub fn as_mint(&self) -> Option<&MintRecord> {
        match self {
            Operation::Mint(record) => Some(record),
            _ => None,
        }
    }

    pub fn is_split(&self) -> bool {
        matches!(self, Operation::Split(_))
    }

    pub fn as_split(&self) -> Option<&SplitRecord> {
        match self {
            Operation::Split(record) => Some(record),
            _ => None,
        }
    }

    pub fn is_merge(&self) -> bool {
        matches!(self, Operation::Merge(_))
    }

    pub fn as_merge(&self) -> Option<&MergeRecord> {
        match self {
            Operation::Merge(record) => Some(record),
            _ => None,
        }
    }

    pub fn op(&self) -> String {
        match self {
            Operation::Deploy(_) => "deploy".to_string(),
            Operation::Mint(_) => "mint".to_string(),
            Operation::Split(_) => "split".to_string(),
            Operation::Merge(_) => "merge".to_string(),
        }
    }
}

pub fn deploy_args_cbor_encode(deploy_args: Vec<String>) -> Vec<u8> {
    let mut mint_args_json: Vec<JSONValue> = vec![];

    for arg in deploy_args.iter() {
        let arg_json: JSONValue =
            serde_json::from_str(arg.as_str()).expect("serde_json unmarshal failed");
        mint_args_json.push(arg_json);
    }

    let mint_args_array = JSONValue::Array(mint_args_json);
    let mut cbor_buffer = Vec::new();
    ciborium::into_writer(&mint_args_array, &mut cbor_buffer).expect("ciborium marshal failed");
    cbor_buffer
}
