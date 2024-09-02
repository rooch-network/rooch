// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use fast_socks5::client::Socks5Stream;
use fast_socks5::server::{Config, Socks5Socket};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};
use move_core_types::language_storage::ModuleId;
use move_core_types::metadata::Metadata;
use move_core_types::resolver::ModuleResolver;
use moveos_types::access_path::AccessPath;
use moveos_types::move_std::string::MoveString;
use moveos_types::moveos_std::move_module::MoveModule;
use moveos_types::state::ObjectState;
use moveos_types::{
    function_return_value::FunctionResult, module_binding::MoveFunctionCaller,
    moveos_std::tx_context::TxContext, transaction::FunctionCall,
};
use pin_project::pin_project;
use rooch_client::RoochRpcClient;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Handle;

pub mod client_config;
pub mod rooch_client;
pub mod wallet_context;

pub struct ClientBuilder {
    request_timeout: Duration,
    ws_url: Option<String>,
    proxy_url: Option<String>,
}

impl ClientBuilder {
    pub fn request_timeout(mut self, request_timeout: Duration) -> Self {
        self.request_timeout = request_timeout;
        self
    }

    pub fn ws_url(mut self, url: impl AsRef<str>) -> Self {
        self.ws_url = Some(url.as_ref().to_string());
        self
    }

    pub fn proxy_url(mut self, url: impl AsRef<str>) -> Self {
        self.proxy_url = Some(url.as_ref().to_string());
        self
    }

    pub async fn build(self, http: impl AsRef<str>) -> Result<Client> {
        // TODO: add verison info

        let http_client = Arc::new(
            HttpClientBuilder::default()
                .max_request_size(2 << 30)
                .request_timeout(self.request_timeout)
                .build(http)?,
        );

        let server_addr = self.socks_server_no_auth().await;
        let server_url = format!("ws://{}", server_addr);
        // TODO: build_with_stream
        let ws_client = Arc::new(WsClientBuilder::default().build(&server_url).await.unwrap());

        Ok(Client {
            http: http_client.clone(),
            ws: ws_client.clone(),
            rooch: RoochRpcClient::new(http_client.clone()),
        })
    }

    pub async fn socks_server_no_auth(self) -> SocketAddr {
        let mut config = Config::default();
        config.set_dns_resolve(false);
        let config = Arc::new(config);

        let proxy_url = if self.proxy_url.is_some() {
            self.proxy_url.clone().unwrap()
        } else {
            env::var("ALL_PROXY").unwrap()
        };

        let listener = TcpListener::bind(proxy_url).await.unwrap();
        let proxy_addr = listener.local_addr().unwrap();
        self.spawn_socks_server(listener, config).await;

        proxy_addr
    }

    pub async fn spawn_socks_server(self, listener: TcpListener, config: Arc<Config>) {
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let mut socks5_socket = Socks5Socket::new(stream, config.clone());
                socks5_socket.set_reply_ip(addr.ip());

                socks5_socket.upgrade_to_socks5().await.unwrap();
            }
        });
    }

    pub async fn connect_over_socks_stream(
        self,
        server_addr: SocketAddr,
    ) -> Socks5Stream<TcpStream> {
        let target_addr = server_addr.ip().to_string();
        let target_port = server_addr.port();

        let socks_server = self.socks_server_no_auth().await;

        Socks5Stream::connect(
            socks_server,
            target_addr,
            target_port,
            fast_socks5::client::Config::default(),
        )
        .await
        .unwrap()
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(60),
            ws_url: None,
            proxy_url: None,
        }
    }
}

#[derive(Clone)]
pub struct Client {
    http: Arc<HttpClient>,
    ws: Arc<WsClient>,
    pub rooch: RoochRpcClient,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RPC client. Http: {:?}", self.http)
    }
}

impl Client {
    pub async fn request(
        &self,
        method: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        Ok(self.http.request(method, params).await?)
    }

    pub async fn request_with_ws(
        &self,
        method: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        Ok(self.ws.request(method, params).await?)
    }
}

impl MoveFunctionCaller for Client {
    fn call_function(
        &self,
        _ctx: &TxContext,
        function_call: FunctionCall,
    ) -> Result<FunctionResult> {
        let function_result =
            futures::executor::block_on(self.rooch.execute_view_function(function_call))?;
        function_result.try_into()
    }
}

impl ModuleResolver for &Client {
    fn get_module_metadata(&self, _module_id: &ModuleId) -> Vec<Metadata> {
        Vec::new()
    }

    fn get_module(&self, id: &ModuleId) -> Result<Option<Vec<u8>>> {
        tokio::task::block_in_place(|| {
            Handle::current().block_on(async {
                let mut states = self.rooch.get_states(AccessPath::module(id)).await?;
                states
                    .pop()
                    .flatten()
                    .map(|state_view| {
                        let state = ObjectState::from(state_view);
                        let module = state.value_as_df::<MoveString, MoveModule>()?;
                        Ok(module.value.byte_codes)
                    })
                    .transpose()
            })
        })
    }
}

#[pin_project]
pub struct DataStream<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin>(
    #[pin] Socks5Stream<T>,
);

impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin> DataStream<T> {
    pub fn new(t: Socks5Stream<T>) -> Self {
        Self(t)
    }
}
