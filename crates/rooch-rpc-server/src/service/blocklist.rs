// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::service::error::ErrorHandler;
use axum::body::Body;
use dashmap::DashMap;
use http::{request::Request, response::Response, StatusCode};
use jsonrpsee::types::ErrorCode;
use pin_project::pin_project;
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{ready, Context, Poll};
use std::time::{Duration, SystemTime};
use tower::{Layer, Service};
use tower_governor::key_extractor::{KeyExtractor, PeerIpKeyExtractor, SmartIpKeyExtractor};
use tower_governor::GovernorError;

type Blocklist = Arc<DashMap<IpAddr, SystemTime>>;
type RejectionMap = Arc<DashMap<IpAddr, Rejection>>;

#[derive(Debug, Clone)]
pub struct BlocklistConfig {
    pub client_rejection_counts: usize,
    pub client_rejection_expiration: Duration,
    pub proxied_rejection_counts: usize,
    pub proxied_rejection_expiration: Duration,
    pub error_handler: ErrorHandler,
    pub clients: Blocklist,
    pub proxied_clients: Blocklist,
    pub rejection_map: RejectionMap,
}

impl Default for BlocklistConfig {
    fn default() -> Self {
        Self {
            client_rejection_counts: 20,
            client_rejection_expiration: Duration::from_secs(60 * 60 * 24),
            proxied_rejection_counts: 60,
            proxied_rejection_expiration: Duration::from_secs(60 * 60 * 6),
            error_handler: Default::default(),
            clients: Arc::new(DashMap::new()),
            proxied_clients: Arc::new(DashMap::new()),
            rejection_map: Arc::new(DashMap::new()),
        }
    }
}

#[derive(Debug)]
pub struct Rejection {
    pub time: SystemTime,
    pub count: usize,
}

// TODO: clear cache
#[derive(Clone)]
pub struct Blocklists<S> {
    pub key_extractor: PeerIpKeyExtractor,
    pub inner: S,
    pub config: BlocklistConfig,
}

impl<S> Blocklists<S> {
    pub fn new(inner: S, config: &BlocklistConfig) -> Self {
        Blocklists {
            inner,
            config: config.clone(),
            key_extractor: PeerIpKeyExtractor,
        }
    }

    pub(crate) fn error_handler(&self) -> &(dyn Fn(GovernorError) -> Response<Body> + Send + Sync) {
        &*self.config.error_handler.0
    }

    /// Returns true if the connection is allowed, false if it is blocked
    pub fn check_impl(&self, client: &Option<IpAddr>, proxied_client: &Option<IpAddr>) -> bool {
        let client_check = self.check_and_clear_blocklist(client, self.config.clients.clone());

        let proxied_client_check =
            self.check_and_clear_blocklist(proxied_client, self.config.proxied_clients.clone());

        client_check && proxied_client_check
    }

    fn check_and_clear_blocklist(&self, client: &Option<IpAddr>, blocklist: Blocklist) -> bool {
        let client = match client {
            Some(client) => client,
            None => return true,
        };
        let now = SystemTime::now();
        // the below two blocks cannot be nested, otherwise we will deadlock
        // due to aquiring the lock on get, then holding across the remove
        let (should_block, should_remove) = {
            match blocklist.get(client) {
                Some(expiration) if now >= *expiration => (false, true),
                None => (false, false),
                _ => (true, false),
            }
        };
        if should_remove {
            blocklist.remove(client);
        }
        !should_block
    }
}

#[derive(Clone)]
pub struct BlockListLayer {
    pub config: Arc<BlocklistConfig>,
}

impl<S> Layer<S> for BlockListLayer {
    type Service = Blocklists<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Blocklists::new(inner, &self.config)
    }
}

impl<S, ReqBody> Service<Request<ReqBody>> for Blocklists<S>
where
    S: Service<Request<ReqBody>, Response = Response<Body>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Our middleware doesn't care about backpressure so its ready as long
        // as the inner service is ready.
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut client: Option<IpAddr> = None;
        let mut proxied_client: Option<IpAddr> = None;
        let ex = SmartIpKeyExtractor;

        // Use the provided key extractor to extract the key from the request.
        match self.key_extractor.extract(&req) {
            Ok(key) => {
                client = Some(key);
            }
            Err(_) => match ex.extract(&req) {
                Ok(key) => {
                    proxied_client = Some(key);
                }
                Err(e) => {
                    let error_response = self.error_handler()(e);
                    return ResponseFuture {
                        inner: Kind::Error {
                            error_response: Some(error_response),
                        },
                    };
                }
            },
        };

        tracing::debug!("requset headers: {:?}", &req.headers());
        tracing::debug!("client ip: {:?}", &client);
        tracing::debug!("proxied client ip: {:?}", &proxied_client);

        let s = self.check_impl(&client, &proxied_client);

        // Extraction worked, let's check blocklist needed.
        if !s {
            let error_response = self.error_handler()(GovernorError::Other {
                code: StatusCode::TOO_MANY_REQUESTS,
                msg: Some(ErrorCode::ServerIsBusy.message().to_string()),
                headers: None,
            });
            ResponseFuture {
                inner: Kind::Error {
                    error_response: Some(error_response),
                },
            }
        } else {
            let future = self.inner.call(req);
            let args = match client {
                None => (
                    Arc::clone(&self.config.proxied_clients),
                    proxied_client.unwrap(),
                    self.config.proxied_rejection_counts,
                    self.config.proxied_rejection_expiration,
                ),

                Some(ip) => (
                    Arc::clone(&self.config.clients),
                    ip,
                    self.config.client_rejection_counts,
                    self.config.client_rejection_expiration,
                ),
            };

            ResponseFuture {
                inner: Kind::Passthrough {
                    future,
                    blocklist: args.0,
                    client: args.1,
                    rejection_map: Arc::clone(&self.config.rejection_map),
                    rejection_count: args.2,
                    rejection_expiration: args.3,
                },
            }
        }
    }
}

#[derive(Debug)]
#[pin_project]
/// Response future.
pub struct ResponseFuture<F> {
    #[pin]
    inner: Kind<F>,
}

#[derive(Debug)]
#[pin_project(project = KindProj)]
enum Kind<F> {
    Passthrough {
        #[pin]
        future: F,
        client: IpAddr,
        blocklist: Blocklist,
        rejection_map: RejectionMap,
        rejection_count: usize,
        rejection_expiration: Duration,
    },
    Error {
        error_response: Option<Response<Body>>,
    },
}

impl<F, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<Body>, E>>,
{
    type Output = Result<Response<Body>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().inner.project() {
            KindProj::Passthrough {
                future,
                client,
                blocklist,
                rejection_map,
                rejection_count,
                rejection_expiration,
            } => {
                let response = ready!(future.poll(cx))?;

                if response.status() == StatusCode::TOO_MANY_REQUESTS {
                    let should_remove;
                    {
                        let mut rejection_entry =
                            rejection_map.entry(*client).or_insert_with(|| Rejection {
                                time: SystemTime::now(),
                                count: 0,
                            });

                        rejection_entry.value_mut().count += 1;
                        should_remove = rejection_entry.value().count > *rejection_count;

                        if should_remove {
                            blocklist.insert(*client, SystemTime::now() + *rejection_expiration);
                        }
                    }

                    if should_remove {
                        rejection_map.remove(client);
                    }
                };

                Poll::Ready(Ok(response))
            }
            KindProj::Error { error_response } => {
                Poll::Ready(Ok(error_response.take().expect("middleware unknown error")))
            }
        }
    }
}
