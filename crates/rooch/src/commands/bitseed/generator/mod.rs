// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::sft::Content;
use bitcoin::{hashes::Hash, Address, BlockHash};
use moveos_types::h256::H256;
use rooch_types::bitcoin::ord::InscriptionID;
use serde::{Deserialize, Serialize};

pub(crate) mod hash;
pub(crate) mod mock;
pub mod wasm;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InscribeGenerateOutput {
    pub amount: u64,
    pub attributes: Option<ciborium::Value>,
    pub content: Option<Content>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct IndexerGenerateOutput {
    pub attributes: Option<ciborium::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InscribeSeed {
    pub utxo: bitcoin::OutPoint,
}

impl InscribeSeed {
    pub fn new(utxo: bitcoin::OutPoint) -> Self {
        Self { utxo }
    }

    pub fn seed(&self) -> H256 {
        let mut buffer = self.utxo.txid.as_byte_array().to_vec();

        let vout_bytes = self.utxo.vout.to_le_bytes();
        buffer.extend_from_slice(&vout_bytes);

        hash::sha3_256_of(buffer.as_slice())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexerSeed {
    pub block_hash: BlockHash,
    pub inscription_id: InscriptionID,
}

impl IndexerSeed {
    pub fn new(block_hash: BlockHash, inscription_id: InscriptionID) -> Self {
        Self {
            block_hash,
            inscription_id,
        }
    }

    pub fn seed(&self) -> H256 {
        let mut buffer = self.block_hash.as_byte_array().to_vec();
        buffer.extend_from_slice(&self.inscription_id.txid.to_vec());
        buffer.extend_from_slice(&self.inscription_id.index.to_le_bytes());
        hash::sha3_256_of(buffer.as_slice())
    }
}

pub const TICK: &str = "generator";
pub const CONTENT_TYPE: &str = "application/wasm";

pub trait Generator {
    fn inscribe_generate(
        &self,
        deploy_args: &[u8],
        seed: &InscribeSeed,
        recipient: &Address,
        user_input: Option<String>,
    ) -> InscribeGenerateOutput;

    fn inscribe_verify(
        &self,
        deploy_args: &[u8],
        seed: &InscribeSeed,
        recipient: &Address,
        user_input: Option<String>,
        inscribe_output: InscribeGenerateOutput,
    ) -> bool;

    fn has_indexer_generate(&self) -> bool {
        false
    }

    fn indexer_generate(
        &self,
        _deploy_args: Vec<u8>,
        _seed: &IndexerSeed,
        _recipient: Address,
    ) -> IndexerGenerateOutput {
        IndexerGenerateOutput::default()
    }
}

pub struct StaticGenerator {
    pub inscribe_output: InscribeGenerateOutput,
    pub indexer_output: Option<IndexerGenerateOutput>,
}

impl StaticGenerator {
    pub fn new(
        inscribe_output: InscribeGenerateOutput,
        indexer_output: Option<IndexerGenerateOutput>,
    ) -> Self {
        Self {
            inscribe_output,
            indexer_output,
        }
    }
}

impl Generator for StaticGenerator {
    fn inscribe_generate(
        &self,
        _deploy_args: &[u8],
        _seed: &InscribeSeed,
        _recipient: &Address,
        _user_input: Option<String>,
    ) -> InscribeGenerateOutput {
        self.inscribe_output.clone()
    }

    fn inscribe_verify(
        &self,
        _deploy_args: &[u8],
        _seed: &InscribeSeed,
        _recipient: &Address,
        _user_input: Option<String>,
        inscribe_output: InscribeGenerateOutput,
    ) -> bool {
        self.inscribe_output == inscribe_output
    }

    fn has_indexer_generate(&self) -> bool {
        self.indexer_output.is_some()
    }

    fn indexer_generate(
        &self,
        _deploy_args: Vec<u8>,
        _seed: &IndexerSeed,
        _recipient: Address,
    ) -> IndexerGenerateOutput {
        self.indexer_output.clone().unwrap()
    }
}
