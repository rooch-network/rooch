use super::segment::Segment;

pub struct Chunker {
    next_segment_id: u64,
    segments: Vec<Segment>,
}