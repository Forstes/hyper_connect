use crate::connection::HttpConnection;
use hyper::body::Body;
use std::future::Future;

pub mod enums;
#[cfg(feature = "http1")]
pub mod http1;
#[cfg(feature = "http1_proxy")]
pub mod http1_proxy;
#[cfg(feature = "http2")]
pub mod http2;
#[cfg(feature = "http2")]
pub mod http2_new;
#[cfg(feature = "http2_proxy")]
pub mod http2_proxy;

pub trait HttpConnector {
    type Connection<B>: HttpConnection<B> + Send + Sync + 'static
    where
        B: Body + Unpin + Send + Sync + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>;

    fn create_connection<B>(&self, uri: &hyper::Uri) -> impl Future<Output = Result<Self::Connection<B>, anyhow::Error>>
    where
        B: Body + 'static + Unpin + Send + Sync,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>;
}
