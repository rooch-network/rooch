use serde::Serialize;

#[derive(Serialize)]
pub struct Segment {
    pub id: SegmentID,
    pub is_last: bool,
    pub data: Vec<u8>,
}

#[derive(Serialize)]
pub struct SegmentID {
    // chunk id represents the sequential order of extents within a stream, commencing from 0 and incrementing successively.
    pub chunk_id: u128,
    pub segment_id: u64,
}
