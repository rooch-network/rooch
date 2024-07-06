// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::fs::File;
use std::io::Result;
#[cfg(target_os = "linux")]
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

#[allow(dead_code)]
pub struct FileCacheManager {
    file: File,
}

impl FileCacheManager {
    pub fn new(file_path: PathBuf) -> Result<Self> {
        let file = File::open(file_path)?;
        Ok(FileCacheManager { file })
    }

    #[cfg(target_os = "linux")]
    pub fn drop_cache_range(&self, offset: u64, len: u64) -> Result<()> {
        let fd = self.file.as_raw_fd();
        let ret = unsafe {
            libc::posix_fadvise(
                fd,
                offset as libc::off_t,
                len as libc::off_t,
                libc::POSIX_FADV_DONTNEED,
            )
        };

        if ret != 0 {
            return Err(std::io::Error::from_raw_os_error(ret));
        }

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn drop_cache_range(&self, _offset: u64, _len: u64) -> Result<()> {
        Ok(())
    }
}
