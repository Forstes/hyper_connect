use hyper::body::{Body, Incoming};
use std::future::Future;

pub trait HttpConnection<B: Body> {
    fn send_request(&mut self, req: hyper::Request<B>) -> impl Future<Output = Result<hyper::Response<Incoming>, hyper::Error>> + Send;
    fn is_conn_ready(&self) -> bool;
    fn is_conn_closed(&self) -> bool;
}

#[cfg(any(feature = "http1", feature = "http1_proxy"))]
pub struct Http1Connection<B: Body + 'static> {
    pub conn: hyper::client::conn::http1::SendRequest<B>,
}

#[cfg(any(feature = "http1", feature = "http1_proxy"))]
impl<B: Body + Send + 'static> HttpConnection<B> for Http1Connection<B> {
    async fn send_request(&mut self, req: hyper::Request<B>) -> Result<hyper::Response<Incoming>, hyper::Error> {
        self.conn.send_request(req).await
    }

    fn is_conn_ready(&self) -> bool {
        self.conn.is_ready()
    }

    fn is_conn_closed(&self) -> bool {
        self.conn.is_closed()
    }
}

#[cfg(any(feature = "http2", feature = "http2_proxy"))]
pub struct Http2Connection<B: Body + 'static> {
    pub conn: hyper::client::conn::http2::SendRequest<B>,
}

#[cfg(any(feature = "http2", feature = "http2_proxy"))]
impl<B: Body + Send + 'static> HttpConnection<B> for Http2Connection<B> {
    async fn send_request(&mut self, req: hyper::Request<B>) -> Result<hyper::Response<Incoming>, hyper::Error> {
        let mut conn = self.conn.clone();
        conn.send_request(req).await
    }

    fn is_conn_ready(&self) -> bool {
        self.conn.is_ready()
    }

    fn is_conn_closed(&self) -> bool {
        self.conn.is_closed()
    }
}
