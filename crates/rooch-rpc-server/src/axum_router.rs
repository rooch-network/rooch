// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::metrics::{ServiceMetrics, TransportProtocol};
use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::Json;
use jsonrpsee::server::RandomIntegerIdProvider;
use jsonrpsee::types::{ErrorCode, ErrorObject, Id, InvalidRequest, Params, Request};
use jsonrpsee::{
    core::server::Methods, BoundedSubscriptions, ConnectionId, MethodCallback, MethodKind,
    MethodResponse, MethodSink,
};
use serde_json::value::RawValue;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::Instant;

pub const MAX_RESPONSE_SIZE: u32 = 2 << 30;

pub const NOT_SUPPORTED_CODE: i32 = 32005;
pub const NOT_SUPPORTED_MSG: &str = "Requests are not supported by this server";

#[derive(Debug, Clone)]
pub(crate) struct CallData<'a> {
    methods: &'a Methods,
    metrics: &'a ServiceMetrics,
    request_start: Instant,
    max_response_body_size: u32,
}

#[derive(Clone, Debug)]
pub struct JsonRpcService {
    /// Registered server methods.
    methods: Methods,
    metrics: ServiceMetrics,
    id_provider: Arc<RandomIntegerIdProvider>,
}

impl JsonRpcService {
    pub fn new(methods: Methods, metrics: ServiceMetrics) -> Self {
        Self {
            methods,
            metrics,
            id_provider: Arc::new(RandomIntegerIdProvider),
        }
    }

    fn call_data(&self) -> CallData<'_> {
        CallData {
            methods: &self.methods,
            metrics: &self.metrics,
            request_start: self.metrics.on_request(TransportProtocol::Http),
            max_response_body_size: MAX_RESPONSE_SIZE,
        }
    }

    fn ws_call_data<'c, 'a: 'c, 'b: 'c>(
        &'a self,
        bounded_subscriptions: BoundedSubscriptions,
        sink: &'b MethodSink,
    ) -> ws::WsCallData<'c> {
        ws::WsCallData {
            metrics: &self.metrics,
            methods: &self.methods,
            max_response_body_size: MAX_RESPONSE_SIZE,
            request_start: self.metrics.on_request(TransportProtocol::Http),
            bounded_subscriptions,
            id_provider: &*self.id_provider,
            sink,
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
        max_response_body_size,
        metrics,
        request_start,
    } = call;

    let params_str = match req.params().parse::<serde_json::Value>() {
        Ok(json) => json.to_string(),
        Err(e) => e.to_string(),
    };

    let conn_id: usize = 0; // unused
    let params = Params::new(req.params.as_ref().map(|params| params.get()));
    let name = &req.method;
    let name_str = name.as_ref();
    let id = req.id;

    tracing::event!(
        tracing::Level::DEBUG,
        event = "on_call",
        method = name_str,
        params = params_str,
    );

    let response = match methods.method_with_name(name) {
        None => {
            metrics.on_call(
                name,
                params.clone(),
                MethodKind::NotFound,
                TransportProtocol::Http,
            );
            MethodResponse::error(id, ErrorObject::from(ErrorCode::MethodNotFound))
        }
        Some((_, method)) => match method {
            MethodCallback::Sync(callback) => {
                metrics.on_call(
                    name,
                    params.clone(),
                    MethodKind::MethodCall,
                    TransportProtocol::Http,
                );
                (callback)(id, params, max_response_body_size as usize, req.extensions)
            }
            MethodCallback::Async(callback) => {
                metrics.on_call(
                    name,
                    params.clone(),
                    MethodKind::MethodCall,
                    TransportProtocol::Http,
                );

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
                metrics.on_call(
                    name,
                    params.clone(),
                    MethodKind::NotFound,
                    TransportProtocol::Http,
                );
                // Subscriptions not supported on HTTP
                MethodResponse::error(id, ErrorObject::from(ErrorCode::InternalError))
            }
        },
    };

    tracing::event!(
        tracing::Level::DEBUG,
        event = "on_result",
        method = name_str,
        result = response.as_result(),
    );

    metrics.on_result(
        name,
        response.is_success(),
        response.as_error_code(),
        request_start,
        TransportProtocol::Http,
    );

    response
}

pub mod ws {
    use super::*;
    use axum::{
        extract::{
            ws::{Message, WebSocket},
            WebSocketUpgrade,
        },
        response::Response,
    };
    use jsonrpsee::{
        core::server::helpers::MethodSink, core::server::BoundedSubscriptions, server::IdProvider,
        types::error::reject_too_many_subscriptions, SubscriptionState,
    };
    use tokio::sync::mpsc;

    #[derive(Debug, Clone)]
    pub(crate) struct WsCallData<'a> {
        pub bounded_subscriptions: BoundedSubscriptions,
        pub id_provider: &'a dyn IdProvider,
        pub methods: &'a Methods,
        pub max_response_body_size: u32,
        pub sink: &'a MethodSink,
        pub metrics: &'a ServiceMetrics,
        pub request_start: Instant,
    }

    // A WebSocket handler that echos any message it receives.
    //
    // This one we'll be integration testing so it can be written in the regular way.
    pub async fn ws_json_rpc_upgrade(
        ws: WebSocketUpgrade,
        State(service): State<JsonRpcService>,
    ) -> Response {
        ws.on_upgrade(|ws| ws_json_rpc_handler(ws, service))
    }

    async fn ws_json_rpc_handler(mut socket: WebSocket, service: JsonRpcService) {
        // #[allow(clippy::disallowed_methods)]
        let (tx, mut rx) = mpsc::channel(100);

        let mut sink = MethodSink::new_with_limit(tx, MAX_RESPONSE_SIZE);
        let bounded_subscriptions = BoundedSubscriptions::new(100);

        loop {
            tokio::select! {
                maybe_message = socket.recv() => {
                    if let Some(Ok(message)) = maybe_message {
                        if let Message::Text(msg) = message {
                            let response =
                                process_raw_request(&service, &msg, bounded_subscriptions.clone(), &sink).await;
                            if let Some(response) = response {
                                let _ = sink.try_send(response.to_result());
                            }
                        }
                    } else {
                        break;
                    }
                },
                Some(response) = rx.recv() => {
                    if socket.send(Message::Text(response)).await.is_err() {
                        break;
                    }
                },
            }
        }
    }

    async fn process_raw_request(
        service: &JsonRpcService,
        raw_request: &str,
        bounded_subscriptions: BoundedSubscriptions,
        sink: &MethodSink,
    ) -> Option<MethodResponse> {
        if let Ok(request) = serde_json::from_str::<Request>(raw_request) {
            process_request(request, service.ws_call_data(bounded_subscriptions, sink)).await
        } else if let Ok(_batch) = serde_json::from_str::<Vec<&RawValue>>(raw_request) {
            Some(MethodResponse::error(
                Id::Null,
                ErrorObject::borrowed(NOT_SUPPORTED_CODE, NOT_SUPPORTED_MSG, None),
            ))
        } else {
            let (id, code) = prepare_error(raw_request);
            Some(MethodResponse::error(id, ErrorObject::from(code)))
        }
    }

    async fn process_request(req: Request<'_>, call: WsCallData<'_>) -> Option<MethodResponse> {
        let WsCallData {
            methods,
            metrics,
            max_response_body_size,
            request_start,
            bounded_subscriptions,
            id_provider,
            sink,
        } = call;
        let conn_id = ConnectionId::from(0u32); // unused

        let params = Params::new(req.params.as_ref().map(|params| params.get()));
        let name = &req.method;
        let id = req.id;

        let response = match methods.method_with_name(name) {
            None => {
                metrics.on_call(
                    name,
                    params.clone(),
                    MethodKind::NotFound,
                    TransportProtocol::Http,
                );
                Some(MethodResponse::error(
                    id,
                    ErrorObject::from(ErrorCode::MethodNotFound),
                ))
            }
            Some((name, method)) => match method {
                MethodCallback::Sync(callback) => {
                    metrics.on_call(
                        name,
                        params.clone(),
                        MethodKind::MethodCall,
                        TransportProtocol::Http,
                    );
                    Some((callback)(
                        id,
                        params,
                        max_response_body_size as usize,
                        req.extensions,
                    ))
                }
                MethodCallback::Async(callback) => {
                    metrics.on_call(
                        name,
                        params.clone(),
                        MethodKind::MethodCall,
                        TransportProtocol::Http,
                    );

                    let id = id.into_owned();
                    let params = params.into_owned();

                    Some(
                        (callback)(
                            id,
                            params,
                            conn_id,
                            max_response_body_size as usize,
                            req.extensions,
                        )
                        .await,
                    )
                }

                MethodCallback::Subscription(callback) => {
                    metrics.on_call(
                        name,
                        params.clone(),
                        MethodKind::Subscription,
                        TransportProtocol::WebSocket,
                    );
                    if let Some(sp) = bounded_subscriptions.acquire() {
                        let conn_state = SubscriptionState {
                            conn_id,
                            subscription_permit: sp,
                            id_provider,
                        };
                        callback(id.clone(), params, sink.clone(), conn_state, req.extensions)
                            .await;
                        None
                    } else {
                        Some(MethodResponse::error(
                            id,
                            reject_too_many_subscriptions(bounded_subscriptions.max()),
                        ))
                    }
                }

                MethodCallback::Unsubscription(callback) => {
                    metrics.on_call(
                        name,
                        params.clone(),
                        MethodKind::Unsubscription,
                        TransportProtocol::WebSocket,
                    );

                    Some(callback(
                        id,
                        params,
                        conn_id,
                        max_response_body_size as usize,
                        req.extensions,
                    ))
                }
            },
        };

        if let Some(response) = &response {
            metrics.on_result(
                name,
                response.is_success(),
                response.as_error_code(),
                request_start,
                TransportProtocol::WebSocket,
            );
        }
        response
    }
}
