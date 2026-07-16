pub use crate::clients::traits::HttpClient;
use crate::{
    clients::request::Request,
    connectors::http1_proxy::ProxyHttp1Connector,
    handlers::{http1::Http1Handler, http1_conn_pool::Http1ConnPool},
};
use hyper::Method;

pub struct Http1ProxyClient {
    handler: Http1Handler<ProxyHttp1Connector>,
}

impl Http1ProxyClient {
    pub fn new(proxy_address: String, username: String, password: String, max_conns_per_host: usize) -> Self {
        let pool = Http1ConnPool::new(ProxyHttp1Connector::new(proxy_address, username, password), max_conns_per_host);
        let handler = Http1Handler::new(pool);
        Self { handler }
    }

    #[cfg(feature = "rate_limit")]
    pub fn new_with_rate_limit(
        max_burst: u64,
        refill_per_sec: f64,
        proxy_address: String,
        username: String,
        password: String,
        max_conns_per_host: usize,
    ) -> Self {
        let pool = Http1ConnPool::new(ProxyHttp1Connector::new(proxy_address, username, password), max_conns_per_host);
        let handler = Http1Handler::new_with_rate_limit(pool, max_burst, refill_per_sec);
        Self { handler }
    }
}

impl HttpClient for Http1ProxyClient {
    type Handler = Http1Handler<ProxyHttp1Connector>;

    fn get<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Handler> {
        Request {
            handler: &self.handler,
            uri,
            method: Method::GET,
            query: String::new(),
            body: None,
            headers: Vec::new(),
            include_host_header: true,
            retry_policy: None,
        }
    }

    fn post<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Handler> {
        Request {
            handler: &self.handler,
            uri,
            method: Method::POST,
            query: String::new(),
            body: None,
            headers: Vec::new(),
            include_host_header: true,
            retry_policy: None,
        }
    }

    fn put<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Handler> {
        Request {
            handler: &self.handler,
            uri,
            method: Method::PUT,
            query: String::new(),
            body: None,
            headers: Vec::new(),
            include_host_header: true,
            retry_policy: None,
        }
    }
}
