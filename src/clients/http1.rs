pub use crate::clients::traits::HttpClient;
use crate::{
    clients::traits::Request,
    connectors::http1::SimpleHttp1Connector,
    handlers::{conn_pool::ConnectionPool, http::HttpHandler},
};
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::Method;

pub struct Http1Client {
    handler: HttpHandler<Either<Full<Bytes>, Empty<Bytes>>, SimpleHttp1Connector>,
}

impl Http1Client {
    pub fn new(max_conns_per_host: usize) -> Self {
        let pool = ConnectionPool::new(SimpleHttp1Connector {}, max_conns_per_host);
        let handler = HttpHandler::new(pool);
        Self { handler }
    }
}

impl HttpClient<SimpleHttp1Connector> for Http1Client {
    fn get<'a>(&'a self, uri: &'a str) -> Request<'a, SimpleHttp1Connector> {
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

    fn post<'a>(&'a self, uri: &'a str) -> Request<'a, SimpleHttp1Connector> {
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
