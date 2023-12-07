// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use celestia_rpc::{BlobClient, Client, StateClient};
use celestia_types::Blob;
use celestia_types::blob::SubmitOptions;
use celestia_types::nmt::Namespace;

use crate::server::segment::Segment;

struct BackendClient {
    namespace: Namespace,
    celestia_client: Client,
}

impl BackendClient {
    async fn new(namespace: Namespace, conn_str: &str, auth_token: &str) -> Self {
        let celestia_client = Client::new(conn_str, Option::from(auth_token)).await?;
        Self { namespace, celestia_client }
    }

    async fn submit(&self, segment: Segment) -> Result<(), String> {
        let data = bcs::to_bytes(&segment).unwrap();
        let blob = Blob::new(self.namespace, data).unwrap();

        // TODO submit to celestia
        self.celestia_client.blob_submit(&[blob.clone()], SubmitOptions::default()).await.unwrap();
        self.celestia_client.state_submit_pay_for_blob()
    }
}