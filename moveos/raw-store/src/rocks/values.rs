// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
use std::marker::PhantomData;

use crate::RawStoreError;
use serde::de::DeserializeOwned;

use super::RocksDBRawIter;

/// An iterator over the values of a prefix.
pub struct Values<'a, V> {
    db_iter: RocksDBRawIter<'a>,
    _phantom: PhantomData<V>,
}

impl<'a, V: DeserializeOwned> Values<'a, V> {
    pub(crate) fn new(db_iter: RocksDBRawIter<'a>) -> Self {
        Self {
            db_iter,
            _phantom: PhantomData,
        }
    }
}

impl<'a, V: DeserializeOwned> Iterator for Values<'a, V> {
    type Item = Result<V, RawStoreError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.db_iter.valid() {
            let value = self
                .db_iter
                .key()
                .and_then(|_| self.db_iter.value().and_then(|v| bcs::from_bytes(v).ok()));

            self.db_iter.next();
            value.map(Ok)
        } else {
            match self.db_iter.status() {
                Ok(_) => None,
                Err(err) => Some(Err(RawStoreError::RocksDBError(format!("{err}")))),
            }
        }
    }
}
