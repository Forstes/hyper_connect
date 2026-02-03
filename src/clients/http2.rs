pub use crate::clients::traits::HttpClient;
use crate::{
    clients::traits::Request,
    connectors::http2::SimpleHttp2Connector,
    handlers::{conn_pool::ConnectionPool, http::HttpHandler},
};
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::Method;

pub struct Http2Client {
    handler: HttpHandler<Either<Full<Bytes>, Empty<Bytes>>, SimpleHttp2Connector>,
}

impl Http2Client {
    pub fn new(max_conns_per_host: usize) -> Self {
        let pool = ConnectionPool::new(SimpleHttp2Connector {}, max_conns_per_host);
        let handler = HttpHandler::new(pool);
        Self { handler }
    }
}

impl HttpClient for Http2Client {
    type Connector = SimpleHttp2Connector;

    fn get<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Connector> {
        Request {
            handler: &self.handler,
            uri,
            method: Method::GET,
            query: String::new(),
            body: None,
            headers: Vec::new(),
            include_host_header: true,
        }
    }

    fn post<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Connector> {
        Request {
            handler: &self.handler,
            uri,
            method: Method::POST,
            query: String::new(),
            body: None,
            headers: Vec::new(),
            include_host_header: true,
        }
    }
}
