use serde::Serialize;

use crate::server::chunk::ChunkID;

#[derive(Serialize)]
pub struct Segment {
    pub id: SegmentID,
    pub is_last: bool,
    pub data: Vec<u8>,
}

#[derive(Serialize)]
pub struct SegmentID {
    pub chunk_id: ChunkID,
    pub segment_id: u64,
}
