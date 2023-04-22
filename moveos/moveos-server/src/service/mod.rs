// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use tonic::{Request, Response as GrpcResponse, Status};

use crate::{
    helper::convert_to_timestamp, os_service_server::OsService, proxy::ServerProxy, HelloRequest,
    HelloResponse, SubmitTransactionRequest, SubmitTransactionResponse,
};

use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::proc_macros::rpc;

use crate::response::JsonResponse;

// Define a rpc server api
#[rpc(server, client)]
trait RpcService {
    #[method(name = "echo")]
    async fn echo(&self, msg: String) -> RpcResult<JsonResponse<String>>;
}

pub struct RoochServer {
    manager: ServerProxy,
}

impl RoochServer {
    pub fn new(manager: ServerProxy) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl RpcServiceServer for RoochServer {
    async fn echo(&self, msg: String) -> RpcResult<JsonResponse<String>> {
        let resp = self.manager.echo(msg).await?;
        Ok(JsonResponse::ok(resp))
    }
}

/// For grpc
pub struct OsSvc {
    manager: ServerProxy,
}

impl OsSvc {
    pub fn new(manager: ServerProxy) -> Self {
        Self { manager }
    }
}

#[tonic::async_trait]
impl OsService for OsSvc {
    async fn echo(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<GrpcResponse<HelloResponse>, Status> {
        let request = request.into_inner();

        let resp = self.manager.echo(request.name).await?;

        let response = HelloResponse {
            message: resp,
            timestamp: Some(convert_to_timestamp(&chrono::Utc::now())),
        };

        Ok(GrpcResponse::new(response))
    }

    async fn submit_txn(
        &self,
        request: Request<SubmitTransactionRequest>,
    ) -> Result<GrpcResponse<SubmitTransactionResponse>, Status> {
        let request = request.into_inner();

        let resp = self.manager.submit_txn(request.txn_payload).await?;

        let response = SubmitTransactionResponse { resp };

        Ok(GrpcResponse::new(response))
    }
}
