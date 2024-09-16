// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use moveos_types::moveos_std::object::ObjectID;
use moveos_types::state::StateChangeSet;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SyncStateFilter {
    /// Sync by object id.
    ObjectID(ObjectID),
    /// Sync all.
    All,
}

// impl SyncStateFilter {
//     fn try_matches(&self, item: &StateChangeSet) -> Result<bool> {
//         Ok(match self {
//             SyncStateFilter::ObjectId(object_id) => object_id == &item.object_id,
//         })
//     }
// }

/// Global State change set ext.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StateChangeSetWithTxOrder {
    pub tx_order: u64,
    pub state_change_set: StateChangeSet,
}

impl StateChangeSetWithTxOrder {
    pub fn new(tx_order: u64, state_change_set: StateChangeSet) -> Self {
        Self {
            tx_order,
            state_change_set,
        }
    }
}
