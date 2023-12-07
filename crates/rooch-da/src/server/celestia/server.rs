// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use celestia_rpc::{BlobClient, Client};
use celestia_types::Blob;
use celestia_types::blob::SubmitOptions;

use crate::messages::{BatchPutRequest, BatchPutResponse};
use crate::server::stream::Stream;

pub struct Server {
    rpc_version: u8,

}

impl Server {
    pub async fn new(conn_str: &str, auth_token: &str) -> Self {
        let celestia_client = Client::new(conn_str, Option::from(auth_token)).await?;
        Self { celestia_client }
    }

    pub fn start() -> Result<()> {
        Ok(())
    }

    // TODO async trait
    pub async fn handle_batch_put(&self, request: BatchPutRequest) -> Result<BatchPutResponse, String> {

        // TODO check version
        // TODO check signature
        // TODO check checksum
        // TODO stream.add_batch(request.batch)
    }
}

impl Stream for Server {
    async fn add_batch(&self, batch: Vec<u8>) -> Result<(), String> {
        let data = batch;
        let blob = Blob::new(self.namespace, data).unwrap();
        self.celestia_client.blob_submit(&[blob.clone()], SubmitOptions::default()).await.unwrap();
    }
}

