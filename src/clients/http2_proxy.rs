pub use crate::clients::traits::HttpClient;
use crate::{
    clients::traits::Request,
    connectors::http2_proxy::ProxyHttp2Connector,
    handlers::{http2::Http2Handler, http2_conn_pool::Http2ConnPool},
};
use hyper::Method;

pub struct Http2ProxyClient {
    handler: Http2Handler<ProxyHttp2Connector>,
}

impl Http2ProxyClient {
    pub fn new(proxy_address: String, username: String, password: String) -> Self {
        let pool = Http2ConnPool::new(ProxyHttp2Connector::new(proxy_address, username, password));
        let handler = Http2Handler::new(pool);
        Self { handler }
    }

    #[cfg(feature = "rate_limit")]
    pub fn new_with_rate_limit(max_burst: u64, refill_per_sec: f64, proxy_address: String, username: String, password: String) -> Self {
        let pool = Http2ConnPool::new(ProxyHttp2Connector::new(proxy_address, username, password));
        let handler = Http2Handler::new_with_rate_limit(pool, max_burst, refill_per_sec);
        Self { handler }
    }
}

impl HttpClient for Http2ProxyClient {
    type Handler = Http2Handler<ProxyHttp2Connector>;

    fn get<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Handler> {
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

    fn post<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Handler> {
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
