// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use rooch_types::bitcoin::ord::{InscriptionID, SatPoint};
use serde::{Deserialize, Serialize};
use std::convert::AsRef;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
    InscriptionCreated {
        block_height: u32,
        charms: u16,
        inscription_id: InscriptionID,
        location: Option<SatPoint>,
        parent_inscription_ids: Vec<InscriptionID>,
        sequence_number: u32,
    },
    InscriptionTransferred {
        block_height: u32,
        inscription_id: InscriptionID,
        new_location: SatPoint,
        old_location: SatPoint,
        sequence_number: u32,
    },
}

/// Load events from a file
/// Every line in the file should be a JSON serialized event
pub fn load_events<P: AsRef<Path>>(path: P) -> Result<Vec<Event>> {
    if !path.as_ref().exists() {
        return Ok(vec![]);
    }
    let file = std::fs::read_to_string(path)?;
    file.lines()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}
