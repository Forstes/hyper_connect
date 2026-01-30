use crate::{connectors::HttpConnector, handlers::http::HttpHandler};
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::{Method, Uri};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::de::SliceRead;

pub trait HttpClient<CN: HttpConnector> {
    fn get<'a>(&'a self, uri: &'a str) -> Request<'a, CN>;
    fn post<'a>(&'a self, uri: &'a str) -> Request<'a, CN>;
}

pub struct Request<'a, CN: HttpConnector> {
    pub(crate) handler: &'a HttpHandler<Either<Full<Bytes>, Empty<Bytes>>, CN>,
    pub(crate) uri: &'a str,
    pub(crate) method: Method,
    pub(crate) query: String,
    pub(crate) body: Option<Vec<u8>>,
    pub(crate) headers: Vec<(&'a str, String)>,
    pub(crate) include_host_header: bool,
}

impl<'a, CN> Request<'a, CN>
where
    CN: HttpConnector,
{
    pub fn query<S: Serialize>(mut self, v: &S) -> Result<Self, serde_urlencoded::ser::Error> {
        self.query = serde_urlencoded::to_string(v)?;
        Ok(self)
    }

    pub fn json<S: Serialize>(mut self, v: &S) -> Result<Self, serde_json::Error> {
        self.body = Some(serde_json::to_vec(v)?);
        self.headers.push(("Content-Type", "application/json".to_string()));
        Ok(self)
    }

    pub fn headers(mut self, mut headers: Vec<(&'a str, String)>) -> Self {
        self.headers.append(&mut headers);
        self
    }

    pub async fn send(self) -> Result<ResponseData, anyhow::Error> {
        let body_data: Either<Full<Bytes>, Empty<Bytes>> = match self.body {
            Some(b) => Either::Left(Full::from(b)),
            None => Either::Right(Empty::<Bytes>::new()),
        };

        let uri = if self.query.is_empty() {
            Uri::try_from(self.uri)?
        } else {
            let mut uri_buf = String::with_capacity(self.uri.len() + 1 + self.query.len());
            uri_buf.push_str(self.uri);
            uri_buf.push('?');
            uri_buf.push_str(&self.query);
            Uri::try_from(uri_buf)?
        };

        let mut builder = hyper::Request::builder()
            .method(&self.method)
            .uri(&uri)
            .header("User-Agent", "RustClient/1.0");

        if self.include_host_header {
            builder = builder.header("HOST", uri.host().unwrap_or_default());
        }

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
