use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
#[cfg(any(feature = "http2", feature = "http2_proxy"))]
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use std::future::Future;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

#[cfg(any(feature = "http1", feature = "http1_proxy"))]
use hyper::client::conn::http1;
#[cfg(any(feature = "http2", feature = "http2_proxy"))]
use hyper::client::conn::http2;

#[cfg(any(feature = "http1", feature = "http1_proxy"))]
pub type Http1Sender = http1::SendRequest<Either<Full<Bytes>, Empty<Bytes>>>;
#[cfg(any(feature = "http1", feature = "http1_proxy"))]
pub type Http1Connection = http1::Connection<TokioIo<TlsStream<TcpStream>>, Either<Full<Bytes>, Empty<Bytes>>>;
#[cfg(any(feature = "http2", feature = "http2_proxy"))]
pub type Http2Sender = http2::SendRequest<Either<Full<Bytes>, Empty<Bytes>>>;
#[cfg(any(feature = "http2", feature = "http2_proxy"))]
pub type Http2Connection = http2::Connection<TokioIo<TlsStream<TcpStream>>, Either<Full<Bytes>, Empty<Bytes>>, TokioExecutor>;

#[cfg(any(feature = "http1", feature = "http1_proxy"))]
pub trait Http1Connector {
    fn create_connection(&self, uri: &hyper::Uri) -> impl Future<Output = Result<(Http1Sender, Http1Connection), anyhow::Error>> + Send;
}

#[cfg(any(feature = "http2", feature = "http2_proxy"))]
pub trait Http2Connector {
    fn create_connection(&self, uri: &hyper::Uri) -> impl Future<Output = Result<(Http2Sender, Http2Connection), anyhow::Error>> + Send;
}
