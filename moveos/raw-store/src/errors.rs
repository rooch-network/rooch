// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Error;

#[derive(thiserror::Error, Debug)]
pub enum RawStoreError {
    #[error("Store check error {0:?}.")]
    StoreCheckError(Error),
}
