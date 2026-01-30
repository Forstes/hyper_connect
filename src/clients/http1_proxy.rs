pub use crate::clients::traits::HttpClient;
use crate::{
    clients::traits::Request,
    connectors::http1_proxy::ProxyHttp1Connector,
    handlers::{conn_pool::ConnectionPool, http::HttpHandler},
};
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::Method;

pub struct Http1ProxyClient {
    handler: HttpHandler<Either<Full<Bytes>, Empty<Bytes>>, ProxyHttp1Connector>,
}

impl Http1ProxyClient {
    pub fn new(proxy_address: String, username: String, password: String, max_conns_per_host: usize) -> Self {
        let pool = ConnectionPool::new(ProxyHttp1Connector::new(proxy_address, username, password), max_conns_per_host);
        let handler = HttpHandler::new(pool);
        Self { handler }
    }
}

impl HttpClient<ProxyHttp1Connector> for Http1ProxyClient {
    fn get<'a>(&'a self, uri: &'a str) -> Request<'a, ProxyHttp1Connector> {
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

    fn post<'a>(&'a self, uri: &'a str) -> Request<'a, ProxyHttp1Connector> {
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
