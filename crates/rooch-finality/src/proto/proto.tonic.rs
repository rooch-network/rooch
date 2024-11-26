// @generated
/// Generated client implementations.
pub mod finality_gadget_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct FinalityGadgetClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl FinalityGadgetClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> FinalityGadgetClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> FinalityGadgetClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            FinalityGadgetClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn query_is_block_babylon_finalized(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryIsBlockBabylonFinalizedRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryIsBlockFinalizedResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/proto.FinalityGadget/QueryIsBlockBabylonFinalized",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "proto.FinalityGadget",
                        "QueryIsBlockBabylonFinalized",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_block_range_babylon_finalized(
            &mut self,
            request: impl tonic::IntoRequest<
                super::QueryBlockRangeBabylonFinalizedRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::QueryBlockRangeBabylonFinalizedResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/proto.FinalityGadget/QueryBlockRangeBabylonFinalized",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "proto.FinalityGadget",
                        "QueryBlockRangeBabylonFinalized",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_btc_staking_activated_timestamp(
            &mut self,
            request: impl tonic::IntoRequest<
                super::QueryBtcStakingActivatedTimestampRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::QueryBtcStakingActivatedTimestampResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/proto.FinalityGadget/QueryBtcStakingActivatedTimestamp",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "proto.FinalityGadget",
                        "QueryBtcStakingActivatedTimestamp",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_is_block_finalized_by_height(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryIsBlockFinalizedByHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryIsBlockFinalizedResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/proto.FinalityGadget/QueryIsBlockFinalizedByHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "proto.FinalityGadget",
                        "QueryIsBlockFinalizedByHeight",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_is_block_finalized_by_hash(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryIsBlockFinalizedByHashRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryIsBlockFinalizedResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/proto.FinalityGadget/QueryIsBlockFinalizedByHash",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "proto.FinalityGadget",
                        "QueryIsBlockFinalizedByHash",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn query_latest_finalized_block(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryLatestFinalizedBlockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryBlockResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/proto.FinalityGadget/QueryLatestFinalizedBlock",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("proto.FinalityGadget", "QueryLatestFinalizedBlock"),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod finality_gadget_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with FinalityGadgetServer.
    #[async_trait]
    pub trait FinalityGadget: std::marker::Send + std::marker::Sync + 'static {
        async fn query_is_block_babylon_finalized(
            &self,
            request: tonic::Request<super::QueryIsBlockBabylonFinalizedRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryIsBlockFinalizedResponse>,
            tonic::Status,
        >;
        async fn query_block_range_babylon_finalized(
            &self,
            request: tonic::Request<super::QueryBlockRangeBabylonFinalizedRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryBlockRangeBabylonFinalizedResponse>,
            tonic::Status,
        >;
        async fn query_btc_staking_activated_timestamp(
            &self,
            request: tonic::Request<super::QueryBtcStakingActivatedTimestampRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryBtcStakingActivatedTimestampResponse>,
            tonic::Status,
        >;
        async fn query_is_block_finalized_by_height(
            &self,
            request: tonic::Request<super::QueryIsBlockFinalizedByHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryIsBlockFinalizedResponse>,
            tonic::Status,
        >;
        async fn query_is_block_finalized_by_hash(
            &self,
            request: tonic::Request<super::QueryIsBlockFinalizedByHashRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryIsBlockFinalizedResponse>,
            tonic::Status,
        >;
        async fn query_latest_finalized_block(
            &self,
            request: tonic::Request<super::QueryLatestFinalizedBlockRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryBlockResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct FinalityGadgetServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> FinalityGadgetServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for FinalityGadgetServer<T>
    where
        T: FinalityGadget,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/proto.FinalityGadget/QueryIsBlockBabylonFinalized" => {
                    #[allow(non_camel_case_types)]
                    struct QueryIsBlockBabylonFinalizedSvc<T: FinalityGadget>(
                        pub Arc<T>,
                    );
                    impl<
                        T: FinalityGadget,
                    > tonic::server::UnaryService<
                        super::QueryIsBlockBabylonFinalizedRequest,
                    > for QueryIsBlockBabylonFinalizedSvc<T> {
                        type Response = super::QueryIsBlockFinalizedResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryIsBlockBabylonFinalizedRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FinalityGadget>::query_is_block_babylon_finalized(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = QueryIsBlockBabylonFinalizedSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/proto.FinalityGadget/QueryBlockRangeBabylonFinalized" => {
                    #[allow(non_camel_case_types)]
                    struct QueryBlockRangeBabylonFinalizedSvc<T: FinalityGadget>(
                        pub Arc<T>,
                    );
                    impl<
                        T: FinalityGadget,
                    > tonic::server::UnaryService<
                        super::QueryBlockRangeBabylonFinalizedRequest,
                    > for QueryBlockRangeBabylonFinalizedSvc<T> {
                        type Response = super::QueryBlockRangeBabylonFinalizedResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryBlockRangeBabylonFinalizedRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FinalityGadget>::query_block_range_babylon_finalized(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = QueryBlockRangeBabylonFinalizedSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/proto.FinalityGadget/QueryBtcStakingActivatedTimestamp" => {
                    #[allow(non_camel_case_types)]
                    struct QueryBtcStakingActivatedTimestampSvc<T: FinalityGadget>(
                        pub Arc<T>,
                    );
                    impl<
                        T: FinalityGadget,
                    > tonic::server::UnaryService<
                        super::QueryBtcStakingActivatedTimestampRequest,
                    > for QueryBtcStakingActivatedTimestampSvc<T> {
                        type Response = super::QueryBtcStakingActivatedTimestampResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryBtcStakingActivatedTimestampRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FinalityGadget>::query_btc_staking_activated_timestamp(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = QueryBtcStakingActivatedTimestampSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/proto.FinalityGadget/QueryIsBlockFinalizedByHeight" => {
                    #[allow(non_camel_case_types)]
                    struct QueryIsBlockFinalizedByHeightSvc<T: FinalityGadget>(
                        pub Arc<T>,
                    );
                    impl<
                        T: FinalityGadget,
                    > tonic::server::UnaryService<
                        super::QueryIsBlockFinalizedByHeightRequest,
                    > for QueryIsBlockFinalizedByHeightSvc<T> {
                        type Response = super::QueryIsBlockFinalizedResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryIsBlockFinalizedByHeightRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FinalityGadget>::query_is_block_finalized_by_height(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = QueryIsBlockFinalizedByHeightSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/proto.FinalityGadget/QueryIsBlockFinalizedByHash" => {
                    #[allow(non_camel_case_types)]
                    struct QueryIsBlockFinalizedByHashSvc<T: FinalityGadget>(pub Arc<T>);
                    impl<
                        T: FinalityGadget,
                    > tonic::server::UnaryService<
                        super::QueryIsBlockFinalizedByHashRequest,
                    > for QueryIsBlockFinalizedByHashSvc<T> {
                        type Response = super::QueryIsBlockFinalizedResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryIsBlockFinalizedByHashRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FinalityGadget>::query_is_block_finalized_by_hash(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = QueryIsBlockFinalizedByHashSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/proto.FinalityGadget/QueryLatestFinalizedBlock" => {
                    #[allow(non_camel_case_types)]
                    struct QueryLatestFinalizedBlockSvc<T: FinalityGadget>(pub Arc<T>);
                    impl<
                        T: FinalityGadget,
                    > tonic::server::UnaryService<
                        super::QueryLatestFinalizedBlockRequest,
                    > for QueryLatestFinalizedBlockSvc<T> {
                        type Response = super::QueryBlockResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::QueryLatestFinalizedBlockRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FinalityGadget>::query_latest_finalized_block(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = QueryLatestFinalizedBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        let mut response = http::Response::new(empty_body());
                        let headers = response.headers_mut();
                        headers
                            .insert(
                                tonic::Status::GRPC_STATUS,
                                (tonic::Code::Unimplemented as i32).into(),
                            );
                        headers
                            .insert(
                                http::header::CONTENT_TYPE,
                                tonic::metadata::GRPC_CONTENT_TYPE,
                            );
                        Ok(response)
                    })
                }
            }
        }
    }
    impl<T> Clone for FinalityGadgetServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    /// Generated gRPC service name
    pub const SERVICE_NAME: &str = "proto.FinalityGadget";
    impl<T> tonic::server::NamedService for FinalityGadgetServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
