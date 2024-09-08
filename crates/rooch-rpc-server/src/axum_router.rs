// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::routing::RpcRouter;
use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::Json;
use jsonrpsee::types::{ErrorCode, ErrorObject, Id, InvalidRequest, Params, Request};
use jsonrpsee::{core::server::Methods, ConnectionId, MethodCallback, MethodResponse};
use serde_json::value::RawValue;
use std::net::SocketAddr;

pub const MAX_RESPONSE_SIZE: u32 = 2 << 30;

pub const NOT_SUPPORTED_CODE: i32 = 32005;
pub const NOT_SUPPORTED_MSG: &str = "Requests are not supported by this server";

#[derive(Debug, Clone)]
pub(crate) struct CallData<'a> {
    methods: &'a Methods,
    // rpc_router: &'a RpcRouter,
    max_response_body_size: u32,
}

#[derive(Clone, Debug)]
pub struct JsonRpcService {
    /// Registered server methods.
    methods: Methods,
    // rpc_router: RpcRouter,
}

impl JsonRpcService {
    pub fn new(methods: Methods, _: RpcRouter) -> Self {
        Self {
            methods,
            // rpc_router,
        }
    }

    fn call_data(&self) -> CallData<'_> {
        CallData {
            methods: &self.methods,
            // rpc_router: &self.rpc_router,
            max_response_body_size: MAX_RESPONSE_SIZE,
        }
    }
}

pub fn from_template<S: Into<axum::body::Body>>(
    status: hyper::StatusCode,
    body: S,
    content_type: &'static str,
) -> Response {
    Response::builder()
        .status(status)
        .header(
            "content-type",
            hyper::header::HeaderValue::from_static(content_type),
        )
        .body(body.into())
        // Parsing `StatusCode` and `HeaderValue` is infalliable but
        // parsing body content is not.
        .expect("Unable to parse response body for type conversion")
}

/// Create a valid JSON response.
pub(crate) fn ok_response(body: String) -> Response {
    const JSON: &str = "application/json; charset=utf-8";
    from_template(hyper::StatusCode::OK, body, JSON)
}

/// Figure out if this is a sufficiently complete request that we can extract an [`Id`] out of, or just plain
/// unparsable garbage.
pub fn prepare_error(data: &str) -> (Id<'_>, ErrorCode) {
    match serde_json::from_str::<InvalidRequest>(data) {
        Ok(InvalidRequest { id }) => (id, ErrorCode::InvalidRequest),
        Err(_) => (Id::Null, ErrorCode::ParseError),
    }
}

pub async fn json_rpc_handler(
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    State(service): State<JsonRpcService>,
    headers: HeaderMap,
    Json(raw_request): Json<Box<RawValue>>,
) -> impl axum::response::IntoResponse {
    let headers_clone = headers.clone();
    // TODO: check request version?

    let response =
        process_raw_request(&service, raw_request.get(), client_addr, headers_clone).await;

    ok_response(response.to_result())
}

async fn process_raw_request(
    service: &JsonRpcService,
    raw_request: &str,
    _: SocketAddr,
    _: HeaderMap,
) -> MethodResponse {
    if let Ok(request) = serde_json::from_str::<Request>(raw_request) {
        let response: MethodResponse = process_request(request, service.call_data()).await;

        response
    } else if let Ok(_batch) = serde_json::from_str::<Vec<&RawValue>>(raw_request) {
        MethodResponse::error(
            Id::Null,
            ErrorObject::borrowed(NOT_SUPPORTED_CODE, NOT_SUPPORTED_MSG, None),
        )
    } else {
        let (id, code) = prepare_error(raw_request);
        MethodResponse::error(id, ErrorObject::from(code))
    }
}

async fn process_request(req: Request<'_>, call: CallData<'_>) -> MethodResponse {
    let CallData {
        methods,
        // rpc_router,
        max_response_body_size,
    } = call;

    let params_str = match req.params().parse::<serde_json::Value>() {
        Ok(json) => json.to_string(),
        Err(e) => e.to_string(),
    };

    tracing::event!(
        tracing::Level::INFO,
        event = "on_call",
        method = req.method_name(),
        params = params_str,
    );

    let conn_id: usize = 0; // unused
    let params = Params::new(req.params.as_ref().map(|params| params.get()));
    let name = &req.method;
    let id = req.id;

    let response = match methods.method_with_name(name) {
        None => MethodResponse::error(id, ErrorObject::from(ErrorCode::MethodNotFound)),
        Some((_, method)) => match method {
            MethodCallback::Sync(callback) => {
                (callback)(id, params, max_response_body_size as usize, req.extensions)
            }
            MethodCallback::Async(callback) => {
                let id = id.into_owned();
                let params = params.into_owned();

                (callback)(
                    id,
                    params,
                    ConnectionId::from(conn_id),
                    max_response_body_size as usize,
                    req.extensions,
                )
                .await
            }
            MethodCallback::Subscription(_) | MethodCallback::Unsubscription(_) => {
                // Subscriptions not supported on HTTP
                MethodResponse::error(id, ErrorObject::from(ErrorCode::InternalError))
            }
        },
    };
    response
}
