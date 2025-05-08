use crate::{
    connectors::http2_new::SimpleHttp2Connector,
    handlers::{conn_pool::ConnectionPool, http_new::HttpHandler},
};
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::{Method, Uri};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::de::SliceRead;

pub struct Http2NewClient {
    handler: HttpHandler<Either<Full<Bytes>, Empty<Bytes>>, SimpleHttp2Connector>,
}

impl Http2NewClient {
    pub fn new(max_conns_per_host: usize) -> Self {
        let pool = ConnectionPool::new(SimpleHttp2Connector {}, max_conns_per_host);
        let handler = HttpHandler::new(pool);
        Self { handler }
    }

    pub fn get<'a>(&'a self, uri: &'a str) -> Request<'a> {
        Request {
            handler: &self.handler,
            uri,
            method: Method::GET,
            headers: Vec::new(),
            body: None,
        }
    }

    pub fn post<'a>(&'a self, uri: &'a str) -> Request<'a> {
        Request {
            handler: &self.handler,
            uri,
            method: Method::POST,
            headers: Vec::new(),
            body: None,
        }
    }
}

pub struct Request<'a> {
    handler: &'a HttpHandler<Either<Full<Bytes>, Empty<Bytes>>, SimpleHttp2Connector>,
    uri: &'a str,
    method: Method,
    headers: Vec<(&'a str, &'a str)>,
    body: Option<Vec<u8>>,
}

impl<'a> Request<'a> {
    pub fn json<S: Serialize>(mut self, v: S) -> Result<Self, serde_json::Error> {
        self.body = Some(serde_json::to_vec(&v)?);
        self.headers.push(("Content-Type", "application/json"));
        Ok(self)
    }

    pub fn headers(mut self, mut headers: Vec<(&'a str, &'a str)>) -> Self {
        self.headers.append(&mut headers);
        self
    }

    pub async fn send(self) -> Result<ResponseData, anyhow::Error> {
        let body_data: Either<Full<Bytes>, Empty<Bytes>> = match self.body {
            Some(b) => Either::Left(Full::from(b)),
            None => Either::Right(Empty::<Bytes>::new()),
        };

        let uri = Uri::try_from(self.uri)?;

        let mut builder = hyper::Request::builder()
            .method(&self.method)
            .uri(&uri)
            .header("User-Agent", "RustClient/1.0");

        for (key, value) in self.headers {
            builder = builder.header(key, value);
        }

        let (status, body) = self.handler.request(&uri, builder.body(body_data)?).await?;

        if status.is_success() {
            Ok(ResponseData { data: body })
        } else {
            Err(anyhow::anyhow!(
                "Request failed with status {}: {}",
                status,
                String::from_utf8_lossy(&body)
            ))
        }
    }
}

pub struct ResponseData {
    pub data: Bytes,
}

impl ResponseData {
    pub fn de<'a>(&'a self) -> serde_json::Deserializer<SliceRead<'a>> {
        serde_json::Deserializer::from_slice(&self.data)
    }

    pub fn to_json<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.data)
    }
}
