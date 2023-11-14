// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::errors::IndexerError;
use crate::types::{IndexedEvent, IndexedTransaction};

pub trait IndexerStoreTrait: Send + Sync {
    fn persist_transactions(
        &self,
        transactions: Vec<IndexedTransaction>,
    ) -> Result<(), IndexerError>;

    fn persist_events(&self, events: Vec<IndexedEvent>) -> Result<(), IndexerError>;

    // fn query_events_with_filter(
    //     &self,
    //     filter: EventFilter,
    //     cursor: Option<IndexerEventID>,
    //     limit: usize,
    //     descending_order: bool,
    // ) -> IndexerResult<Vec<IndexerEvent>>;
}
