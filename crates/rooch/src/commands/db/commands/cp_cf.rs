// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::utils::open_inner_rocks;
use clap::Parser;
use rocksdb::{WriteBatch, DB};
use rooch_types::error::RoochResult;

/// copy column family by column family name
#[derive(Debug, Parser)]
pub struct CpCfCommand {
    #[clap(
        long = "cf-name",
        help = "column family name, if not set, copy all column families"
    )]
    pub cf_name: Option<String>,
    #[clap(long = "dst", help = "destination path to new rocksdb")]
    pub dst: String,
    #[clap(long = "src", help = "source path to rocksdb")]
    pub src: String,
}

impl CpCfCommand {
    pub fn execute(self) -> RoochResult<()> {
        let cf_names = if let Some(cf_name) = self.cf_name.clone() {
            vec![cf_name]
        } else {
            let mut column_families = moveos_store::StoreMeta::get_column_family_names().to_vec();
            column_families.append(&mut rooch_store::StoreMeta::get_column_family_names().to_vec());
            column_families.into_iter().map(String::from).collect()
        };

        let source_db = open_inner_rocks(&self.src, cf_names.clone(), true)?;
        let target_db = open_inner_rocks(&self.dst, cf_names.clone(), false)?;
        let migrator = DBMigrator {
            source_db,
            target_db,
            batch_size: 1048576,
            cf_names,
        };
        migrator.migrate()?;
        Ok(())
    }
}

struct DBMigrator {
    source_db: DB,
    target_db: DB,
    batch_size: usize,
    cf_names: Vec<String>,
}

impl DBMigrator {
    pub fn migrate(&self) -> anyhow::Result<()> {
        for cf_name in self.cf_names.iter() {
            let source_cf = self.source_db.cf_handle(cf_name).unwrap();
            let target_cf = self.target_db.cf_handle(cf_name).unwrap();

            let mut batch = WriteBatch::default();
            let mut count = 0;

            let iter = self
                .source_db
                .iterator_cf(source_cf, rocksdb::IteratorMode::Start);

            for result in iter {
                let (key, value) = result?;
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
        }

        Ok(())
    }
}
