// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use tonic::{Request, Response as GrpcResponse, Status};

use crate::{
    helper::convert_to_timestamp, os_service_server::OsService, proxy::ServerProxy,
    ExecutionFunctionRequest, ExecutionFunctionResponse, HelloRequest, HelloResponse,
    PublishPackageRequest, PublishPackageResponse,
};

use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::proc_macros::rpc;

use crate::response::JsonResponse;

// Define a rpc server api
#[rpc(server, client)]
trait RpcService {
    #[method(name = "echo")]
    async fn echo(&self, msg: String) -> RpcResult<JsonResponse<String>>;

    #[method(name = "publish")]
    async fn publish(&self, module_bytes: Vec<u8>) -> RpcResult<JsonResponse<String>>;

    #[method(name = "execute_function")]
    async fn execute_function(&self, function_bytes: Vec<u8>) -> RpcResult<JsonResponse<String>>;
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

    async fn publish(&self, module_bytes: Vec<u8>) -> RpcResult<JsonResponse<String>> {
        let resp = self.manager.publish(module_bytes).await?;
        Ok(JsonResponse::ok(resp))
    }

    async fn execute_function(&self, function_bytes: Vec<u8>) -> RpcResult<JsonResponse<String>> {
        let resp = self.manager.execute_function(function_bytes).await?;
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

    async fn publish(
        &self,
        request: Request<PublishPackageRequest>,
    ) -> Result<GrpcResponse<PublishPackageResponse>, Status> {
        let request = request.into_inner();

        let resp = self.manager.publish(request.module).await?;

        let response = PublishPackageResponse { resp };

        Ok(GrpcResponse::new(response))
    }

    async fn execute_function(
        &self,
        request: Request<ExecutionFunctionRequest>,
    ) -> Result<GrpcResponse<ExecutionFunctionResponse>, Status> {
        let request = request.into_inner();

        let resp = self.manager.execute_function(request.functions).await?;

        let response = ExecutionFunctionResponse { resp };

        Ok(GrpcResponse::new(response))
    }
}
