// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::messages::Batch;
use crate::segment::{Segment, SegmentID, SegmentV0};
use lz4::EncoderBuilder;
use serde::{Deserialize, Serialize};

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
    fn to_bytes(&self) -> Vec<u8>;
    fn get_version(&self) -> ChunkVersion;
    fn to_segments(&self, max_segment_size: usize) -> Vec<Box<dyn Segment>>;
}

// ChunkV0:
// 1. each chunk maps to a batch
// 2. batch_data compressed by lz4
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ChunkV0 {
    pub version: ChunkVersion,
    pub batch: Batch,
}

impl From<Batch> for ChunkV0 {
    fn from(batch: Batch) -> Self {
        Self {
            version: ChunkVersion::V0,
            batch: Batch {
                block_number: batch.block_number,
                tx_count: batch.tx_count,
                prev_tx_accumulator_root: batch.prev_tx_accumulator_root,
                tx_accumulator_root: batch.tx_accumulator_root,
                batch_hash: batch.batch_hash,
                data: batch.data,
            },
        }
    }
}

impl Chunk for ChunkV0 {
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

    fn get_version(&self) -> ChunkVersion {
        ChunkVersion::V0
    }

    fn to_segments(&self, max_segment_size: usize) -> Vec<Box<dyn Segment>> {
        let bytes = self.to_bytes();
        let segments_data = bytes.chunks(max_segment_size);
        let segments_count = segments_data.len();

        let chunk_id = self.batch.block_number;
        segments_data
            .enumerate()
            .map(|(i, data)| {
                Box::new(SegmentV0 {
                    id: SegmentID {
                        chunk_id,
                        segment_number: i as u64,
                    },
                    is_last: i == segments_count - 1, // extra info overhead is much smaller than max_block_size - max_segment_size
                    // *_checksum will be filled in to_bytes method of Segment
                    data_checksum: 0,
                    checksum: 0,
                    data: data.to_vec(),
                }) as Box<dyn Segment>
            })
            .collect::<Vec<_>>()
    }
}
