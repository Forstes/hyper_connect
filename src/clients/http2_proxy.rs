pub use crate::clients::traits::HttpClient;
use crate::{
    clients::traits::Request,
    connectors::http2_proxy::ProxyHttp2Connector,
    handlers::{conn_pool::ConnectionPool, http::HttpHandler},
};
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::Method;

pub struct Http2ProxyClient {
    handler: HttpHandler<Either<Full<Bytes>, Empty<Bytes>>, ProxyHttp2Connector>,
}

impl Http2ProxyClient {
    pub fn new(proxy_address: String, username: String, password: String, max_conns_per_host: usize) -> Self {
        let pool = ConnectionPool::new(ProxyHttp2Connector::new(proxy_address, username, password), max_conns_per_host);
        let handler = HttpHandler::new(pool);
        Self { handler }
    }
}

impl HttpClient for Http2ProxyClient {
    type Connector = ProxyHttp2Connector;

    fn get<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Connector> {
        Request {
            handler: &self.handler,
            uri,
            method: Method::GET,
            query: String::new(),
            body: None,
            headers: Vec::new(),
            include_host_header: false,
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
            include_host_header: false,
        }
    }
}
