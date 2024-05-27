// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::{
    borrow::Cow,
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    str::FromStr,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tower::limit::RateLimitLayer;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use crate::{DiscordConfig, FaucetError, FaucetRequest, FaucetResponse, InfoResponse};

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
use rooch_rpc_api::jsonrpc_types::StructTagView;
use rooch_rpc_client::wallet_context::WalletContext;
use tokio::sync::RwLock;

pub const METRICS_ROUTE: &str = "/metrics";

const CONCURRENCY_LIMIT: usize = 10;

#[derive(Parser, Debug, Clone)]
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
    pub faucet_queue: Sender<FaucetRequest>,
    pub err_receiver: Arc<RwLock<Receiver<FaucetError>>>,
    pub wallet_config_dir: Option<PathBuf>,
    pub discord_config: DiscordConfig,
    pub faucet_funds: u64,
    pub is_loop_running: Arc<AtomicBool>,
}

impl App {
    pub fn new(
        faucet_queue: Sender<FaucetRequest>,
        wallet_config_dir: Option<PathBuf>,
        discord_config: DiscordConfig,
        err_receiver: Receiver<FaucetError>,
        faucet_funds: u64,
    ) -> Self {
        Self {
            faucet_queue,
            wallet_config_dir,
            discord_config,
            faucet_funds,
            is_loop_running: Arc::new(AtomicBool::new(false)),
            err_receiver: Arc::new(RwLock::new(err_receiver)),
        }
    }

    pub async fn request(&self, address: FaucetRequest) -> Result<(), FaucetError> {
        self.faucet_queue
            .send(address)
            .await
            .map_err(FaucetError::internal)?;
        Ok(())
    }

    pub async fn check_gas_balance(&self) -> Result<f64, FaucetError> {
        let context = WalletContext::new(self.wallet_config_dir.clone())
            .map_err(|e| FaucetError::Wallet(e.to_string()))?;
        let client = context.get_client().await.unwrap();
        let faucet_address = context.client_config.active_address.unwrap();

        let s = client
            .rooch
            .get_balance(
                faucet_address.into(),
                StructTagView::from_str("0x3::gas_coin::GasCoin").unwrap(),
            )
            .await
            .map_err(FaucetError::internal)?;

        let divisor: u64 = 10u64.pow(s.coin_info.decimals as u32);
        let result = s.balance.0.unchecked_as_u64() as f64 / divisor as f64;

        Ok(result)
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
        .route("/info", get(request_info))
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

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), app_config.port);

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
            tracing::info!("request gas success add queue: {}", recipient);
            (
                StatusCode::CREATED,
                Json(FaucetResponse::from(app.faucet_funds.to_string())),
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

pub async fn request_info(Extension(app): Extension<App>) -> impl IntoResponse {
    let result = app.check_gas_balance().await;

    match result {
        Ok(v) => (StatusCode::OK, Json(InfoResponse::from(v))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(InfoResponse::from(e)),
        ),
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
