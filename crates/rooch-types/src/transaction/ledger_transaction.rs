// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{RoochTransaction, TransactionSequenceInfo};
use crate::{
    address::RoochAddress,
    multichain_id::{MultiChainID, RoochMultiChainID},
};
use accumulator::accumulator_info::AccumulatorInfo;
use anyhow::Result;
use bitcoin::hashes::Hash;
use core::fmt;
use moveos_types::h256::H256;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
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
        bcs::serialized_size(self).expect("serialize transaction size should success") as u64
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct L1BlockWithBody {
    pub block: L1Block,
    pub block_body: Vec<u8>,
}

impl L1BlockWithBody {
    pub fn new(block: L1Block, block_body: Vec<u8>) -> Self {
        Self { block, block_body }
    }

    pub fn new_bitcoin_block(height: u64, block: bitcoin::Block) -> Self {
        let block_hash = block.block_hash();
        let block_body = crate::bitcoin::types::Block::from(block);
        let l1_block = L1Block {
            chain_id: RoochMultiChainID::Bitcoin.multichain_id(),
            block_height: height,
            block_hash: block_hash.to_byte_array().to_vec(),
        };
        Self {
            block: l1_block,
            block_body: block_body.encode(),
        }
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct L1Transaction {
    pub chain_id: MultiChainID,
    pub block_hash: Vec<u8>,
    /// The original L1 transaction id, usually the hash of the transaction
    pub txid: Vec<u8>,
}

impl fmt::Debug for L1Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let block_hash = if self.chain_id.is_bitcoin() {
            bitcoin::BlockHash::from_slice(&self.block_hash)
                .map(|hash| hash.to_string())
                .unwrap_or("invalid block hash".to_string())
        } else {
            hex::encode(&self.block_hash)
        };
        let txid = if self.chain_id.is_bitcoin() {
            bitcoin::Txid::from_slice(&self.txid)
                .map(|hash| hash.to_string())
                .unwrap_or("invalid txid".to_string())
        } else {
            hex::encode(&self.txid)
        };
        write!(
            f,
            "L1Transaction {{ chain_id: {:?}, block_hash: {}, txid: {} }}",
            self.chain_id, block_hash, txid
        )
    }
}

impl L1Transaction {
    pub fn new(chain_id: MultiChainID, block_hash: Vec<u8>, txid: Vec<u8>) -> Self {
        Self {
            chain_id,
            block_hash,
            txid,
        }
    }

    pub fn tx_hash(&self) -> H256 {
        moveos_types::h256::sha3_256_of(self.encode().as_slice())
    }

    pub fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("encode transaction should success")
    }

    pub fn tx_size(&self) -> u64 {
        bcs::serialized_size(self).expect("serialize transaction size should success") as u64
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum LedgerTxData {
    L1Block(L1Block),
    L1Tx(L1Transaction),
    L2Tx(RoochTransaction),
}

impl LedgerTxData {
    pub fn tx_hash(&mut self) -> H256 {
        match self {
            LedgerTxData::L1Block(block) => block.tx_hash(),
            LedgerTxData::L2Tx(tx) => tx.tx_hash(),
            LedgerTxData::L1Tx(tx) => tx.tx_hash(),
        }
    }

    pub fn sender(&self) -> Option<RoochAddress> {
        match self {
            LedgerTxData::L1Block(_) => None,
            LedgerTxData::L2Tx(tx) => Some(tx.sender()),
            LedgerTxData::L1Tx(_) => None,
        }
    }

    pub fn is_l1_block(&self) -> bool {
        matches!(self, LedgerTxData::L1Block(_))
    }

    pub fn is_l1_tx(&self) -> bool {
        matches!(self, LedgerTxData::L1Tx(_))
    }

    pub fn is_l2_tx(&self) -> bool {
        matches!(self, LedgerTxData::L2Tx(_))
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

    pub fn tx_hash(&mut self) -> H256 {
        self.data.tx_hash()
    }

    pub fn sender(&self) -> Option<RoochAddress> {
        self.data.sender()
    }

    pub fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("encode transaction should success")
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }

    pub fn build_ledger_transaction(
        tx_data: LedgerTxData,
        tx_timestamp: u64,
        tx_order: u64,
        tx_order_signature: Vec<u8>,
        tx_accumulator_info: AccumulatorInfo,
    ) -> LedgerTransaction {
        let tx_sequence_info = TransactionSequenceInfo::new(
            tx_order,
            tx_order_signature,
            tx_accumulator_info,
            tx_timestamp,
        );

        LedgerTransaction::new(tx_data, tx_sequence_info)
    }
}
