use super::json_http;
use crate::{
    connectors::{enums::connector::ConnectorEnum, http2::SimpleHttp2Connector},
    handlers::http::HttpHandler,
};
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::{Method, Uri};
use serde::de::DeserializeOwned;
use serde_json::Value;
pub struct Http2Client {
    handler: HttpHandler<Either<Full<Bytes>, Empty<Bytes>>>,
}

impl Http2Client {
    pub fn new() -> Self {
        let connector = SimpleHttp2Connector {};
        let handler = HttpHandler::new(ConnectorEnum::Http2(connector));
        Self { handler }
    }

    pub async fn request_many<T: DeserializeOwned>(
        &mut self,
        uris: &Vec<Uri>,
        method: Method,
        bodies: &Vec<Option<Value>>,
    ) -> Result<Vec<(Option<T>, Option<anyhow::Error>)>, anyhow::Error> {
        json_http::request_batch(&mut self.handler, uris, method, bodies, false).await
    }

    pub async fn get<T: DeserializeOwned>(
        &mut self,
        uri: &str,
        params: Option<Value>,
        headers: Option<Vec<(String, String)>>,
    ) -> Result<T, anyhow::Error> {
        let uri = json_http::build_uri_with_params(uri, params)?;
        json_http::request(&mut self.handler, &uri, Method::GET, None, headers, false).await
    }

    pub async fn post<T: DeserializeOwned>(
        &mut self,
        uri: &str,
        body: Option<Value>,
        headers: Option<Vec<(String, String)>>,
    ) -> Result<T, anyhow::Error> {
        json_http::request(&mut self.handler, uri, Method::POST, body, headers, false).await
    }

    pub async fn put<T: DeserializeOwned>(
        &mut self,
        uri: &str,
        body: Option<Value>,
        headers: Option<Vec<(String, String)>>,
    ) -> Result<T, anyhow::Error> {
        json_http::request(&mut self.handler, uri, Method::PUT, body, headers, false).await
    }

    pub async fn delete<T: DeserializeOwned>(
        &mut self,
        uri: &str,
        headers: Option<Vec<(String, String)>>,
    ) -> Result<T, anyhow::Error> {
        json_http::request(&mut self.handler, &uri, Method::DELETE, None, headers, false).await
    }
}
