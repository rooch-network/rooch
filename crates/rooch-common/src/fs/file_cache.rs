// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::io::Result;
use std::path::PathBuf;

pub struct FileCacheManager {
    #[cfg(target_os = "linux")]
    file: std::fs::File,
}

impl FileCacheManager {
    #[cfg(target_os = "linux")]
    pub fn new(file_path: PathBuf) -> Result<Self> {
        let file = std::fs::File::open(file_path)?;
        Ok(FileCacheManager { file })
    }

    #[cfg(target_os = "linux")]
    pub fn drop_cache_range(&self, offset: u64, len: u64) -> Result<()> {
        use std::os::unix::io::AsRawFd;

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
    pub fn new(_: PathBuf) -> Result<Self> {
        Ok(FileCacheManager {})
    }

    #[cfg(not(target_os = "linux"))]
    pub fn drop_cache_range(&self, _: u64, _: u64) -> Result<()> {
        Ok(())
    }
}
