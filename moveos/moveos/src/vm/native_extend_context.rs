// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use better_any::Tid;
use moveos_types::bitcoin_client::BitcoinClient;

/// The native bitcoin light client context.
#[derive(Tid)]
pub struct NativeBitcoinLightClientContext<'a> {
    bitcoin_client: &'a Option<BitcoinClient>,
}

impl<'a> NativeBitcoinLightClientContext<'a> {
    /// Create a new instance of a native bitcoin light client context. This must be passed in via an
    /// extension into VM session functions.
    pub fn new(bitcoin_client: &'a Option<BitcoinClient>) -> Self {
        Self { bitcoin_client }
    }
}
