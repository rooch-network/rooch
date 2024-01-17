// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::str::FromStr;

use serde::Serialize;

#[derive(Serialize, Debug, PartialEq)]
pub struct Segment {
    pub id: SegmentID,
    pub is_last: bool,
    pub data: Vec<u8>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct SegmentID {
    // chunk id represents the sequential order of extents within a stream, commencing from 0 and incrementing successively.
    pub chunk_id: u128,
    pub segment_id: u64,
}

impl fmt::Display for SegmentID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.chunk_id, self.segment_id)
    }
}

impl FromStr for SegmentID {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err("Invalid string format for SegmentID");
        }

        let chunk_id = u128::from_str(parts[0]).map_err(|_| "Invalid chunk_id")?;
        let segment_id = u64::from_str(parts[1]).map_err(|_| "Invalid segment_id")?;

        Ok(SegmentID {
            chunk_id,
            segment_id,
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
            segment_id: 456,
        };

        let segment_id_str = segment_id.to_string();
        assert_eq!(segment_id_str, "123-456");

        let parsed_segment_id: SegmentID = segment_id_str.parse().unwrap();
        assert_eq!(parsed_segment_id, segment_id);
    }
}
