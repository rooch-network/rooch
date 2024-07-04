// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use libc::{posix_fadvise, POSIX_FADV_DONTNEED};
use std::fs::File;
use std::io::{Error, Result};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

pub struct FileCacheManager {
    file: File,
}

impl FileCacheManager {
    pub fn new(file_path: PathBuf) -> Result<Self> {
        let file = File::open(file_path)?;
        Ok(FileCacheManager { file })
    }

    pub fn drop_cache_range(&self, offset: u64, len: u64) -> Result<()> {
        let fd = self.file.as_raw_fd();
        let ret = unsafe {
            posix_fadvise(
                fd,
                offset as libc::off_t,
                len as libc::off_t,
                POSIX_FADV_DONTNEED,
            )
        };

        if ret != 0 {
            return Err(Error::from_raw_os_error(ret));
        }

        Ok(())
    }
}
