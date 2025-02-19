// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_inner_rocks;
use clap::Parser;
use rocksdb::{WriteBatch, DB};
use rooch_types::error::RoochResult;

/// copy column family by column family name
#[derive(Debug, Parser)]
pub struct CpCfCommand {
    #[clap(long = "cf-name")]
    pub cf_name: String,
    #[clap(long = "dst", help = "destination path to new rocksdb")]
    pub dst: String,
    #[clap(long = "src", help = "source path to rocksdb")]
    pub src: String,
}

impl CpCfCommand {
    pub fn execute(self) -> RoochResult<()> {
        let source_db = open_inner_rocks(&self.src, vec![self.cf_name.clone()], true)?;
        let target_db = open_inner_rocks(&self.dst, vec![self.cf_name.clone()], false)?;
        let migrator = DBMigrator {
            source_db,
            target_db,
            batch_size: 1048576,
            cf_name: self.cf_name.clone(),
        };
        migrator.migrate()?;
        Ok(())
    }
}

struct DBMigrator {
    source_db: DB,
    target_db: DB,
    batch_size: usize,
    cf_name: String,
}

impl DBMigrator {
    pub fn migrate(&self) -> anyhow::Result<()> {
        let cf_name = self.cf_name.clone();

        let source_cf = self.source_db.cf_handle(&cf_name).unwrap();
        let target_cf = self.target_db.cf_handle(&cf_name).unwrap();

        let mut batch = WriteBatch::default();
        let mut count = 0;

        let mut iter = self
            .source_db
            .iterator_cf(source_cf, rocksdb::IteratorMode::Start);

        loop {
            let (key, value) = match iter.next() {
                Some(result) => result?,
                None => break,
            };

            batch.put_cf(&target_cf, key, value);
            count += 1;

            if count % self.batch_size == 0 {
                self.target_db.write(batch)?;
                batch = WriteBatch::default();
                tracing::info!("{} records migrated", count);
            }
        }

        if !batch.is_empty() {
            self.target_db.write(batch)?;
        }

        self.target_db.flush()?;

        Ok(())
    }
}
