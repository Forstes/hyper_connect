pub use crate::clients::traits::HttpClient;
use crate::{
    clients::traits::Request,
    connectors::http2::SimpleHttp2Connector,
    handlers::{http2::Http2Handler, http2_conn_pool::Http2ConnPool},
};
use hyper::Method;

pub struct Http2Client {
    handler: Http2Handler<SimpleHttp2Connector>,
}

impl Http2Client {
    pub fn new() -> Self {
        let pool = Http2ConnPool::new(SimpleHttp2Connector {});
        let handler = Http2Handler::new(pool);
        Self { handler }
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
