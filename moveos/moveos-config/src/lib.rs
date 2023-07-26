// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

pub mod store_config;

#[derive(Clone, Debug)]
pub enum DataDirPath {
    PathBuf(PathBuf),
    TempPath(Arc<TempDir>),
}

pub fn temp_dir() -> DataDirPath {
    let temp_dir = TempDir::new().expect("Create temp dir fail.");
    DataDirPath::TempPath(Arc::from(temp_dir))
}

pub fn temp_dir_in(dir: PathBuf) -> DataDirPath {
    let temp_dir = TempDir::new_in(dir).expect("Create temp dir fail.");
    DataDirPath::TempPath(Arc::from(temp_dir))
}

impl DataDirPath {
    pub fn path(&self) -> &Path {
        self.as_ref()
    }
    pub fn is_temp(&self) -> bool {
        matches!(self, DataDirPath::TempPath(_))
    }
}

impl AsRef<Path> for DataDirPath {
    fn as_ref(&self) -> &Path {
        match self {
            DataDirPath::PathBuf(path) => path.as_ref(),
            DataDirPath::TempPath(path) => path.as_ref().as_ref(),
        }
    }
}
