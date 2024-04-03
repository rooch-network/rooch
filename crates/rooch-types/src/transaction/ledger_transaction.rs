// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{RoochTransaction, TransactionSequenceInfo};
use crate::multichain_id::MultiChainID;
use anyhow::Result;
use ethers::types::H256;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct L1Block {
    pub chain_id: MultiChainID,
    pub block_height: u64,
    pub block_hash: Vec<u8>,
}

impl L1Block {
    pub fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("encode transaction should success")
    }

    pub fn tx_hash(&self) -> H256 {
        moveos_types::h256::sha3_256_of(self.encode().as_slice())
    }

    pub fn tx_size(&self) -> u64 {
        //TODO optimize the size calculation
        self.encode().len() as u64
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct L1BlockWithBody {
    pub block: L1Block,
    pub block_body: Vec<u8>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum LedgerTxData {
    L1Block(L1Block),
    L2Tx(RoochTransaction),
}

impl LedgerTxData {
    pub fn tx_hash(&self) -> H256 {
        match self {
            LedgerTxData::L1Block(block) => block.tx_hash(),
            LedgerTxData::L2Tx(tx) => tx.tx_hash(),
        }
    }
}

/// The transaction which is recorded in the L2 DA ledger.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LedgerTransaction {
    pub data: LedgerTxData,
    pub sequence_info: TransactionSequenceInfo,
}

impl LedgerTransaction {
    pub fn new(data: LedgerTxData, sequence_info: TransactionSequenceInfo) -> Self {
        Self {
            data,
            sequence_info,
        }
    }

    pub fn new_l1_block(
        chain_id: MultiChainID,
        block_height: u64,
        block_hash: Vec<u8>,
        sequence_info: TransactionSequenceInfo,
    ) -> Self {
        Self {
            data: LedgerTxData::L1Block(L1Block {
                chain_id,
                block_height,
                block_hash,
            }),
            sequence_info,
        }
    }

    pub fn new_l2_tx(tx: RoochTransaction, sequence_info: TransactionSequenceInfo) -> Self {
        Self {
            data: LedgerTxData::L2Tx(tx),
            sequence_info,
        }
    }

    pub fn tx_hash(&self) -> H256 {
        self.data.tx_hash()
    }

    pub fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("encode transaction should success")
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }
}
