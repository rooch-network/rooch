// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::da::batch::DABatch;
use crate::da::segment::{Segment, SegmentID, SegmentV0};
use lz4::EncoderBuilder;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ChunkVersion {
    V0,
    Unknown(u8),
}

impl From<u8> for ChunkVersion {
    fn from(num: u8) -> Self {
        match num {
            0 => ChunkVersion::V0,
            // ...
            _ => Self::Unknown(num),
        }
    }
}

impl From<ChunkVersion> for u8 {
    fn from(version: ChunkVersion) -> Self {
        match version {
            ChunkVersion::V0 => 0,
            ChunkVersion::Unknown(num) => num,
        }
    }
}

pub trait Chunk {
    fn get_version(&self) -> ChunkVersion;
    fn to_segments(&self, max_segment_size: usize) -> Vec<Box<dyn Segment>>;
    fn get_batches(&self) -> Vec<DABatch>;
    fn get_chunk_id(&self) -> u128;
}

// ChunkV0:
// 1. each chunk maps to a batch (block number is chunk_id)
// 2. batch_data compressed by lz4
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ChunkV0 {
    pub version: ChunkVersion,
    pub batch: DABatch,
}

impl From<DABatch> for ChunkV0 {
    fn from(batch: DABatch) -> Self {
        Self {
            version: ChunkVersion::V0,
            batch,
        }
    }
}

impl ChunkV0 {
    fn to_bytes(&self) -> Vec<u8> {
        let mut compressed_bytes = Vec::new();

        {
            let mut encoder = EncoderBuilder::new().build(&mut compressed_bytes).unwrap();
            bcs::serialize_into(&mut encoder, self).unwrap();
            let (_output, result) = encoder.finish();
            result.unwrap();
        }

        compressed_bytes
    }
}

impl Chunk for ChunkV0 {
    fn get_version(&self) -> ChunkVersion {
        ChunkVersion::V0
    }

    fn to_segments(&self, max_segment_size: usize) -> Vec<Box<dyn Segment>> {
        let bytes = self.to_bytes();
        let segments_data = bytes.chunks(max_segment_size);
        let segments_count = segments_data.len();

        let chunk_id = self.get_chunk_id();
        segments_data
            .enumerate()
            .map(|(i, data)| {
                Box::new(SegmentV0 {
                    id: SegmentID {
                        chunk_id,
                        segment_number: i as u64,
                    },
                    is_last: i == segments_count - 1, // extra info overhead is much smaller than max_block_size - max_segment_size
                    data_len: data.len() as u64,
                    // *_checksum will be filled in to_bytes method of Segment
                    data_checksum: 0,
                    checksum: 0,
                    data: data.to_vec(),
                }) as Box<dyn Segment>
            })
            .collect::<Vec<_>>()
    }

    fn get_batches(&self) -> Vec<DABatch> {
        vec![self.batch.clone()]
    }

    /// using batch.meta.block_number as chunk_id
    fn get_chunk_id(&self) -> u128 {
        self.batch.meta.block_range.block_number
    }
}

pub fn chunk_from_segments(segments: Vec<Box<dyn Segment>>) -> anyhow::Result<Box<dyn Chunk>> {
    if segments.is_empty() {
        return Err(anyhow::anyhow!("empty segments"));
    }
    // check all segments have the same version
    let versions = segments
        .iter()
        .map(|segment| segment.get_version())
        .collect::<Vec<_>>();
    let version = versions.first().unwrap();
    if versions.iter().any(|seg_version| *seg_version != *version) {
        return Err(anyhow::anyhow!("inconsistent segment versions"));
    }
    // check last segment.is_last == true, others must be false
    if let Some(last_segment) = segments.last() {
        if last_segment.is_last() {
            if segments
                .iter()
                .take(segments.len() - 1)
                .any(|segment| segment.is_last())
            {
                return Err(anyhow::anyhow!("inconsistent is_last"));
            }
        } else {
            return Err(anyhow::anyhow!("missing last segments"));
        }
    }
    // check all segments have the same chunk_id, segment_number starts from 0 and increments by 1
    let chunk_id = segments.first().unwrap().get_id().chunk_id;
    if segments.iter().enumerate().any(|(i, segment)| {
        segment.get_id()
            != SegmentID {
                chunk_id,
                segment_number: i as u64,
            }
    }) {
        return Err(anyhow::anyhow!("inconsistent segment ids"));
    }

    match version {
        ChunkVersion::V0 => Ok(Box::new(ChunkV0::from_segments(segments)?)),
        // ...
        ChunkVersion::Unknown(_) => Err(anyhow::anyhow!("unsupported segment version")),
    }
}

impl ChunkV0 {
    pub fn from_segments(segments: Vec<Box<dyn Segment>>) -> anyhow::Result<Self> {
        let bytes = segments
            .iter()
            .flat_map(|segment| segment.get_data())
            .collect::<Vec<_>>();

        let decoder = lz4::Decoder::new(&bytes[..])?;
        let mut decompressed_reader = io::BufReader::new(decoder);
        let chunk: ChunkV0 = bcs::from_reader(&mut decompressed_reader)?;
        Ok(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::RoochKeyPair;
    use crate::test_utils::random_ledger_transaction;
    use crate::transaction::LedgerTransaction;

    #[test]
    fn test_chunk_v0() {
        let tx_cnt = 128;
        let keypair = RoochKeyPair::generate_secp256k1();

        let tx_list = (0..tx_cnt)
            .map(|_| random_ledger_transaction())
            .collect::<Vec<_>>();
        let batch = DABatch::new(123, 56, 78, &tx_list, keypair);

        let chunk = ChunkV0::from(batch.clone());
        let segments = chunk.to_segments(1023);

        let chunk = chunk_from_segments(segments).unwrap();
        let batches = chunk.get_batches();
        let act_batch = batches.first().unwrap();
        assert_eq!(act_batch, &batch);

        let act_tx_list: Vec<LedgerTransaction> =
            bcs::from_bytes(&act_batch.tx_list_bytes).expect("decode tx_list should success");
        assert_eq!(tx_list, act_tx_list);

        assert!(act_batch.verify(false).is_ok())
    }
}
