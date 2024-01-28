// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::str::FromStr;

use serde::Serialize;

#[derive(Debug, PartialEq, Eq)]
pub enum SegmentVersion {
    V0,
    Unknown(u8),
}

impl From<u8> for SegmentVersion {
    fn from(num: u8) -> Self {
        match num {
            0 => Self::V0,
            // ...
            _ => Self::Unknown(num),
        }
    }
}

// `Segment` is the basic unit of storage in DA server.
pub trait Segment: Send {
    fn to_bytes(&self) -> Vec<u8>;
    fn get_version(&self) -> SegmentVersion;
    fn get_id(&self) -> SegmentID;
}

pub const SEGMENT_V0_DATA_OFFSET: usize = 42;

#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct SegmentV0 {
    pub id: SegmentID,
    pub is_last: bool,      // is last segment in chunk
    pub data_checksum: u64, // checksum of data, xxh3_64
    pub checksum: u64,      // checksum of above fields(exclude data), xxh3_64

    pub data: Vec<u8>,
}

impl SegmentV0 {
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        if bytes.len() < SEGMENT_V0_DATA_OFFSET {
            return Err(anyhow::anyhow!("segment_v0: bytes less than {}", SEGMENT_V0_DATA_OFFSET));
        }

        let chunk_id = u128::from_le_bytes(bytes[1..17].try_into()?);
        let segment_number = u64::from_le_bytes(bytes[17..25].try_into()?);
        let is_last = bytes[25] != 0;
        let data_checksum = u64::from_le_bytes(bytes[26..34].try_into()?);
        let checksum = u64::from_le_bytes(bytes[34..SEGMENT_V0_DATA_OFFSET].try_into()?);
        let data = bytes[SEGMENT_V0_DATA_OFFSET..].to_vec();

        Ok(Self {
            id: SegmentID {
                chunk_id,
                segment_number,
            },
            is_last,
            data_checksum,
            checksum,
            data,
        })
    }
}

impl Segment for SegmentV0 {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(SEGMENT_V0_DATA_OFFSET + self.data.len());

        bytes.push(0);
        bytes.extend_from_slice(&self.id.chunk_id.to_le_bytes());
        bytes.extend_from_slice(&self.id.segment_number.to_le_bytes());
        bytes.push(self.is_last as u8);
        bytes.extend_from_slice(&self.data_checksum.to_le_bytes());
        bytes.extend_from_slice(&self.checksum.to_le_bytes());
        bytes.extend_from_slice(&self.data);
        bytes
    }

    fn get_version(&self) -> SegmentVersion {
        SegmentVersion::V0
    }

    fn get_id(&self) -> SegmentID {
        self.id.clone()
    }
}

pub fn get_data_offset(version: SegmentVersion) -> usize {
    match version {
        SegmentVersion::V0 => SEGMENT_V0_DATA_OFFSET,
        SegmentVersion::Unknown(_) => panic!("unsupported segment version"),
    }
}

// falling back to Result here to cater for corrupted data etc
pub fn segment_from_bytes(bytes: &[u8]) -> anyhow::Result<Box<dyn Segment>> {
    let version = bytes[0];

    match SegmentVersion::from(version) {
        SegmentVersion::V0 => Ok(Box::new(SegmentV0::from_bytes(bytes)?)),
        // ...
        SegmentVersion::Unknown(_) => Err(anyhow::anyhow!("unsupported segment version")),
    }
}

#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct SegmentID {
    // chunk id represents the sequential order of extents within a stream, commencing from 0 and incrementing successively.
    pub chunk_id: u128,
    // segment number within chunk, commencing from 0 and incrementing successively.
    pub segment_number: u64,
}

impl fmt::Display for SegmentID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.chunk_id, self.segment_number)
    }
}

impl FromStr for SegmentID {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('_').collect();
        if parts.len() != 2 {
            return Err("invalid string format for segment_id");
        }

        let chunk_id = u128::from_str(parts[0]).map_err(|_| "invalid chunk_id")?;
        let segment_id = u64::from_str(parts[1]).map_err(|_| "invalid segment_id")?;

        Ok(SegmentID {
            chunk_id,
            segment_number: segment_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_id_display_and_from_str() {
        let segment_id = SegmentID {
            chunk_id: 123,
            segment_number: 456,
        };

        let segment_id_str = segment_id.to_string();
        assert_eq!(segment_id_str, "123_456");

        let parsed_segment_id: SegmentID = segment_id_str.parse().unwrap();
        assert_eq!(parsed_segment_id, segment_id);
    }

    #[test]
    fn test_segment_trait() {
        let segment_v0 = SegmentV0 {
            id: SegmentID {
                chunk_id: 1234567890,
                segment_number: 12345678,
            },
            is_last: true,
            data_checksum: 1234567890,
            checksum: 12345678,
            data: vec![1, 2, 3, 4, 5],
        };

        let segments: Vec<Box<dyn Segment>> = vec![Box::new(segment_v0.clone())];

        for segment in segments {
            let bytes = segment.to_bytes();
            let version = segment.get_version();

            match version {
                SegmentVersion::V0 => {
                    let recovered_segment =
                        SegmentV0::from_bytes(&bytes).expect("successful deserialization");
                    assert_eq!(&segment_v0, &recovered_segment)
                }

                _ => panic!("unsupported segment version"),
            };
        }
    }
}
