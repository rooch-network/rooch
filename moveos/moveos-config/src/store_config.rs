// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Parser)]
#[serde(default, deny_unknown_fields)]
pub struct RocksdbConfig {
    #[clap(name = "rocksdb-max-open-files", long, help = "rocksdb max open files")]
    pub max_open_files: i32,
    #[clap(
        name = "rocksdb-max-total-wal-sizes",
        long,
        help = "rocksdb max total WAL sizes"
    )]
    pub max_total_wal_size: u64,
    #[clap(
        name = "rocksdb-wal-bytes-per-sync",
        long,
        help = "rocksdb wal bytes per sync"
    )]
    pub wal_bytes_per_sync: u64,
    #[clap(name = "rocksdb-bytes-per-sync", long, help = "rocksdb bytes per sync")]
    pub bytes_per_sync: u64,
    #[clap(
        name = "rocksdb-max-background-jobs",
        long,
        help = "rocksdb max background jobs"
    )]
    pub max_background_jobs: u64,
    #[clap(name = "rocksdb-row-cache-size", long, help = "rocksdb row cache size")]
    pub row_cache_size: u64,
    #[clap(
        name = "rocksdb-max-write-buffer-number",
        long,
        help = "rocksdb max write buffer number"
    )]
    pub max_write_buffer_numer: u64,

    // row cache is expensive (not so memory efficient), we need block cache at the same time.
    // share large block cache between column families, dozens GB is required.
    #[clap(
        name = "rocksdb-block-cache-size",
        long,
        help = "rocksdb block cache size"
    )]
    pub block_cache_size: u64,
    #[clap(name = "rocksdb-block-size", long, help = "rocksdb block size")]
    pub block_size: u64,
}

impl RocksdbConfig {
    #[cfg(unix)]
    fn default_max_open_files() -> i32 {
        4096
    }

    #[cfg(windows)]
    fn default_max_open_files() -> i32 {
        256
    }
}

impl Default for RocksdbConfig {
    fn default() -> Self {
        Self {
            max_open_files: Self::default_max_open_files(),
            max_total_wal_size: 1u64 << 31,
            bytes_per_sync: 1u64 << 20,
            max_background_jobs: 6, // rocksdb will decide compaction threads and flush threads based on this number
            wal_bytes_per_sync: 1u64 << 20,
            row_cache_size: 1u64 << 27,
            max_write_buffer_numer: 4,
            block_cache_size: 1u64 << 31,
            block_size: 1u64 << 10,
        }
    }
}
