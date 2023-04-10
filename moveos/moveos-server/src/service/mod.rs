// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use tonic::{async_trait, Request, Response, Status};

use crate::{
    helper::convert_to_timestamp, os_service_server::OsService, proxy::ServerProxy, HelloRequest,
    HelloResponse,
};

pub struct OsSvc {
    manager: ServerProxy,
}

impl OsSvc {
    pub fn new(manager: ServerProxy) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl OsService for OsSvc {
    async fn echo(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let request = request.into_inner();

        let resp = self.manager.echo(request.name).await?;

        let response = HelloResponse {
            message: resp,
            timestamp: Some(convert_to_timestamp(&chrono::Utc::now())),
        };

        Ok(Response::new(response))
    }
}
