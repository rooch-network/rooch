// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::segment::Segment;

pub struct Chunker {
    next_segment_id: u64,
    segments: Vec<Segment>,
}
