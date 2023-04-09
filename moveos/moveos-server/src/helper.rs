// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use chrono::{DateTime, Utc};

use prost_types::Timestamp;

// Convert chrono DateTime to prost_type Timestamp
pub fn convert_to_timestamp(dt: &DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as _,
    }
}
