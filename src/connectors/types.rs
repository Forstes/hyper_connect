use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
#[cfg(any(feature = "http1", feature = "http1_proxy"))]
use hyper::client::conn::http1;
#[cfg(any(feature = "http2", feature = "http2_proxy"))]
use hyper::client::conn::http2;

#[cfg(any(feature = "http1", feature = "http1_proxy"))]
pub type Http1Connection = http1::SendRequest<Either<Full<Bytes>, Empty<Bytes>>>;
#[cfg(any(feature = "http2", feature = "http2_proxy"))]
pub type Http2Connection = http2::SendRequest<Either<Full<Bytes>, Empty<Bytes>>>;

#[cfg(any(feature = "http2", feature = "http2_proxy"))]
pub trait Http2Connector {
    async fn create_connection(&self, uri: &hyper::Uri) -> Result<Http2Connection, anyhow::Error>;
}
