use anyhow::Result;
use clap::Parser;
use moveos::types::transaction::{SimpleTransaction, ViewPayload};
use moveos_common::config::load_config;
use moveos_server::{
    os_service_client::OsServiceClient, SubmitTransactionRequest, ViewFunctionRequest,
};
use tokio::time::Duration;
use tonic::transport::Channel;

#[derive(Clone, Debug, Parser)]
pub struct Client {
    #[clap(long)]
    rpc: Option<String>,
}

impl Client {
    pub fn connect(&self) -> Result<()> {
        self.connect_with_timeout(Duration::from_secs(30))
    }

    pub fn connect_with_timeout(&self, timeout: Duration) -> Result<()> {
        // TODO: connect to rpc server
        Ok(())
    }

    // get mutable OsServiceClient
    async fn rpc_client(&self) -> Result<OsServiceClient<Channel>> {
        let url = match self.rpc.clone() {
            Some(url) => url,
            None => load_config()?.server.url(false),
        };

        OsServiceClient::connect(url)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn submit(&self, txn: SimpleTransaction) -> Result<()> {
        let txn_payload = bcs::to_bytes(&txn)?;

        let request = SubmitTransactionRequest { txn_payload };
        let _resp = self.rpc_client().await?.submit_txn(request).await?;
        // TODO: parse response.
        Ok(())
    }

    pub async fn view(&self, payload: ViewPayload) -> Result<()> {
        let payload = bcs::to_bytes(&payload)?;
        let request = ViewFunctionRequest { payload };
        let _resp = self.rpc_client().await?.view(request).await?;
        // TODO: parse response.
        Ok(())
    }
}
