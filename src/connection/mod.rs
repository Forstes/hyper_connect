use hyper::body::{Body, Incoming};
use std::future::Future;

pub trait HttpConnection<B: Body> {
    fn send_request(
        &mut self,
        req: hyper::Request<B>,
    ) -> impl Future<Output = Result<hyper::Response<Incoming>, hyper::Error>> + Send;
    fn is_conn_ready(&self) -> bool;
    fn is_conn_closed(&self) -> bool;
}

#[cfg(feature = "http2")]
mod http2;
#[cfg(feature = "http2")]
pub use http2::Http2Connection;
