// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::{open_inner_rocks, TxSizeHist};
use clap::Parser;
use rooch_store::STATE_CHANGE_SET_COLUMN_FAMILY_NAME;
use rooch_types::error::RoochResult;

/// Get changeset by order, helping to decide how to save changeset,
/// if a majority of changesets is small, we can save it in one batch with other execution data.
#[derive(Debug, Parser)]
pub struct StatChangesetCommand {
    #[clap(long = "src", help = "source path to rocksdb")]
    pub src: String,
}

impl StatChangesetCommand {
    pub async fn execute(self) -> RoochResult<()> {
        let cf_name = STATE_CHANGE_SET_COLUMN_FAMILY_NAME.to_string();
        let db = open_inner_rocks(&self.src, vec![cf_name.clone()], true)?;
        let source_cf = db.cf_handle(&cf_name).unwrap();
        let iter = db.iterator_cf(source_cf, rocksdb::IteratorMode::Start);

        const TOP_N: usize = 20;

        let mut hist = TxSizeHist::new("Changeset".to_string(), TOP_N, None, None)?;

        for result in iter {
            let (key, value) = result.expect("Failed to get key-value");
            let tx_order: u64 = bcs::from_bytes(&key)?;
            hist.record(tx_order, value.len() as u64)?;
        }

        hist.print();
        Ok(())
    }
}
