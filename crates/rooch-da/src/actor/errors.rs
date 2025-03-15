// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#[derive(thiserror::Error, Debug)]
pub enum SubmitBatchError {
    #[error("Database is inconsistent: {0}")]
    DatabaseInconsistent(anyhow::Error),

    #[error("Recoverable submission error: {0}")]
    Recoverable(anyhow::Error),
}
