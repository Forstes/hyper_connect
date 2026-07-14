pub use crate::clients::{request::RetryPolicy, traits::HttpClient};
use crate::{
    clients::request::Request,
    connectors::http2::SimpleHttp2Connector,
    handlers::{http2::Http2Handler, http2_conn_pool::Http2ConnPool},
};
use hyper::Method;

pub struct Http2Client {
    handler: Http2Handler<SimpleHttp2Connector>,
    retry_policy: Option<RetryPolicy>,
}

impl Http2Client {
    pub fn new() -> Self {
        let pool = Http2ConnPool::new(SimpleHttp2Connector {});
        let handler = Http2Handler::new(pool);
        Self { handler, retry_policy: None }
    }

    #[cfg(feature = "rate_limit")]
    pub fn new_with_rate_limit(max_burst: u64, refill_per_sec: f64) -> Self {
        let pool = Http2ConnPool::new(SimpleHttp2Connector {});
        let handler = Http2Handler::new_with_rate_limit(pool, max_burst, refill_per_sec);
        Self { handler, retry_policy: None }
    }

    #[cfg(feature = "rate_limit")]
    pub fn new_with_rate_limit_and_retry(max_burst: u64, refill_per_sec: f64, retry_policy: RetryPolicy) -> Self {
        let pool = Http2ConnPool::new(SimpleHttp2Connector {});
        let handler = Http2Handler::new_with_rate_limit(pool, max_burst, refill_per_sec);
        Self {
            handler,
            retry_policy: Some(retry_policy),
        }
    }
}

impl HttpClient for Http2Client {
    type Handler = Http2Handler<SimpleHttp2Connector>;

    fn get<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Handler> {
        Request {
            handler: &self.handler,
            uri,
            method: Method::GET,
            query: String::new(),
            body: None,
            headers: Vec::new(),
            include_host_header: false,
            retry_policy: self.retry_policy.clone(),
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
            retry_policy: self.retry_policy.clone(),
        }
    }
}
