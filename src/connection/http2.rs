use super::HttpConnection;
use hyper::body::{Body, Incoming};

pub struct Http2Connection<B: Body + 'static> {
    pub conn: hyper::client::conn::http2::SendRequest<B>,
}

impl<B: Body + Send + 'static> HttpConnection<B> for Http2Connection<B> {
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
