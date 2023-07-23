// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use serde::{Deserialize, Serialize};

/// Port selected RocksDB options for tuning underlying rocksdb instance of DiemDB.
/// see https://github.com/facebook/rocksdb/blob/master/include/rocksdb/options.h
/// for detailed explanations.
/// https://github.com/facebook/rocksdb/wiki/WAL-Performance
/// wal_bytes_per_sync, bytes_per_sync see https://github.com/facebook/rocksdb/wiki/IO#range-sync
/// for detailed explanations.
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
}

impl RocksdbConfig {
    #[cfg(unix)]
    fn default_max_open_files() -> i32 {
        40960
    }

    #[cfg(windows)]
    fn default_max_open_files() -> i32 {
        256
    }
}

impl Default for RocksdbConfig {
    fn default() -> Self {
        Self {
            // Set max_open_files to 4096 instead of -1 to avoid keep-growing memory in accordance
            // with the number of files.
            max_open_files: Self::default_max_open_files(),
            // For now we set the max total WAL size to be 1G. This config can be useful when column
            // families are updated at non-uniform frequencies.
            max_total_wal_size: 1u64 << 30,
            // For sst table sync every size to be 1MB
            bytes_per_sync: 1u64 << 20,
            // For wal sync every size to be 1MB
            wal_bytes_per_sync: 1u64 << 20,
        }
    }
}