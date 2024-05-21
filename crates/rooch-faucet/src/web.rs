// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::{
    borrow::Cow,
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::Duration,
};
use tokio::sync::mpsc::Sender;
use tower::limit::RateLimitLayer;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use crate::{FaucetError, FaucetRequest, FaucetResponse};
use rooch_types::address::BitcoinAddress;

use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    BoxError, Extension, Json, Router,
};
use clap::Parser;
use http::Method;
use prometheus::{Registry, TextEncoder};

pub const METRICS_ROUTE: &str = "/metrics";

const CONCURRENCY_LIMIT: usize = 10;

// const PROM_PORT_ADDR: &str = "0.0.0.0:9184";

#[derive(Parser, Clone)]
#[clap(rename_all = "kebab-case")]
pub struct AppConfig {
    #[clap(long, default_value_t = 50052)]
    pub port: u16,

    #[clap(long, default_value_t = 10)]
    pub request_buffer_size: usize,

    #[clap(long, default_value_t = 10)]
    pub max_request_per_second: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 50052,
            request_buffer_size: 10,
            max_request_per_second: 10,
        }
    }
}

#[derive(Clone, Debug)]
pub struct App {
    faucet_queue: Sender<FaucetRequest>,
}

impl App {
    pub fn new(faucet_queue: Sender<FaucetRequest>) -> Self {
        Self { faucet_queue }
    }

    pub async fn request(&self, address: FaucetRequest) -> Result<(), FaucetError> {
        self.faucet_queue
            .send(address)
            .await
            .map_err(FaucetError::internal)?;
        Ok(())
    }
}

pub async fn serve(app: App, app_config: AppConfig) -> Result<(), anyhow::Error> {
    let max_concurrency = match env::var("MAX_CONCURRENCY") {
        Ok(val) => val.parse::<usize>().unwrap(),
        _ => CONCURRENCY_LIMIT,
    };

    // TODO:: 分开跑
    // let prom_binding = PROM_PORT_ADDR.parse().unwrap();
    // info!("Starting Prometheus HTTP endpoint at {}", prom_binding);

    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    let router = Router::new()
        .route("/", get(health))
        .route(METRICS_ROUTE, get(metrics))
        .route("/gas", post(request_gas))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                // .layer(RequestMetricsLayer::new(&registry))
                .layer(cors)
                .load_shed()
                .buffer(app_config.request_buffer_size)
                .layer(RateLimitLayer::new(
                    app_config.max_request_per_second,
                    Duration::from_secs(1),
                ))
                .concurrency_limit(max_concurrency)
                .layer(Extension(app))
                .into_inner(),
        );

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), app_config.port);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

pub async fn metrics(registry: Extension<Registry>) -> (StatusCode, String) {
    let metrics_families = registry.gather();
    match TextEncoder.encode_to_string(&metrics_families) {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("unable to encode metrics: {error}"),
        ),
    }
}

async fn request_gas(
    Extension(app): Extension<App>,
    Json(payload): Json<FaucetRequest>,
) -> impl IntoResponse {
    let recipient = payload.recipient().to_string();

    tracing::info!("request gas payload: {:?}", recipient);

    if let FaucetRequest::FixedETHAddressRequest(_) = payload {
        tracing::warn!("request gas with ETH: {:?}", recipient);
        return (
            StatusCode::BAD_REQUEST,
            Json(FaucetResponse::from(FaucetError::NotSupport(
                "ETH".to_string(),
            ))),
        );
    }

    let result = app.request(payload).await;

    match result {
        Ok(()) => {
            tracing::info!("request gas success: {}", recipient);
            (
                StatusCode::CREATED,
                Json(FaucetResponse::from("Success".to_string())),
            )
        }
        Err(e) => {
            tracing::info!("request gas error: {}, {:?}", recipient, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(FaucetResponse::from(e)),
            )
        }
    }
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Cow::from("service is overloaded, please try again later"),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from(format!("Unhandled internal error: {}", error)),
    )
}
