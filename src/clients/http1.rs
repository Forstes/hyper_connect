use hyper::Method;

pub use crate::clients::traits::HttpClient;
use crate::{
    clients::request::Request,
    connectors::http1::SimpleHttp1Connector,
    handlers::{http1::Http1Handler, http1_conn_pool::Http1ConnPool},
};

pub struct Http1Client {
    handler: Http1Handler<SimpleHttp1Connector>,
}

impl Http1Client {
    pub fn new(max_conns_per_host: usize) -> Self {
        let pool = Http1ConnPool::new(SimpleHttp1Connector {}, max_conns_per_host);
        let handler = Http1Handler::new(pool);
        Self { handler }
    }

    #[cfg(feature = "rate_limit")]
    pub fn new_with_rate_limit(max_burst: u64, refill_per_sec: f64, max_conns_per_host: usize) -> Self {
        let pool = Http1ConnPool::new(SimpleHttp1Connector {}, max_conns_per_host);
        let handler = Http1Handler::new_with_rate_limit(pool, max_burst, refill_per_sec);
        Self { handler }
    }
}

impl HttpClient for Http1Client {
    type Handler = Http1Handler<SimpleHttp1Connector>;

    fn get<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Handler> {
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

    fn post<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Handler> {
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
