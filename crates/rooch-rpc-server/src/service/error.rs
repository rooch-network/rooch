// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::axum_router::from_template;
use axum::body::Body;
use axum::response::Response;
use http::StatusCode;
use jsonrpsee::types::{ErrorCode, ErrorObject, Id};
use jsonrpsee::MethodResponse;
use std::fmt;
use std::sync::Arc;
use tower_governor::GovernorError;

const TOO_MANY_REQUESTS_MSG: &str = "Too many requests! Wait for ";

#[derive(Clone)]
pub struct ErrorHandler(pub(crate) Arc<dyn Fn(GovernorError) -> Response<Body> + Send + Sync>);

impl Default for ErrorHandler {
    fn default() -> Self {
        Self(Arc::new(|e| {
            let result = match e {
                GovernorError::TooManyRequests { headers, wait_time } => (
                    ErrorObject::owned(
                        ErrorCode::ServerIsBusy.code(),
                        format!("{}{}s", TOO_MANY_REQUESTS_MSG, wait_time),
                        None::<bool>,
                    ),
                    headers,
                    StatusCode::TOO_MANY_REQUESTS,
                ),
                GovernorError::UnableToExtractKey => (
                    ErrorObject::borrowed(
                        ErrorCode::InvalidRequest.code(),
                        ErrorCode::InvalidRequest.message(),
                        None,
                    ),
                    None,
                    StatusCode::FORBIDDEN,
                ),
                GovernorError::Other { code, msg, headers } => (
                    ErrorObject::owned(
                        ErrorCode::ServerIsBusy.code(),
                        msg.unwrap_or_else(|| ErrorCode::InternalError.message().to_string()),
                        None::<bool>,
                    ),
                    headers,
                    code,
                ),
            };

            let rpc_resp = MethodResponse::error(Id::Null, result.0).to_result();
            const JSON: &str = "application/json; charset=utf-8";
            let mut resp = from_template(result.2, rpc_resp, JSON);

            if let Some(headers) = result.1 {
                for (key, value) in headers.iter() {
                    resp.headers_mut().insert(key, value.clone());
                }
            }

            resp
        }))
    }
}

impl fmt::Debug for ErrorHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErrorHandler").finish()
    }
}

impl PartialEq for ErrorHandler {
    fn eq(&self, _: &Self) -> bool {
        // there is no easy way to tell two object equals.
        true
    }
}

impl Eq for ErrorHandler {}
