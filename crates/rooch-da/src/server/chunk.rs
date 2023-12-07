use super::segment::Segment;

// chunk id represents the sequential order of extents within a stream, commencing from 0 and incrementing successively.
pub type ChunkID = u64;

pub struct Chunker {
    next_segment_id: u64,
    segments: Vec<Segment>,
}