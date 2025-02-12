use super::json_http;
use crate::{
    connectors::{enums::connector::ConnectorEnum, http2_proxy::ProxyHttp2Connector},
    handlers::http::HttpHandler,
};
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::{Method, Uri};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

pub struct Http2ProxyClient {
    handler: HttpHandler<Either<Full<Bytes>, Empty<Bytes>>>,
}

impl Http2ProxyClient {
    pub fn new(proxy_address: String, username: String, password: String) -> Self {
        let connector = ProxyHttp2Connector::new(proxy_address, username, password);
        let handler = HttpHandler::new(ConnectorEnum::Http2Proxy(connector));
        Self { handler }
    }

    pub async fn request_many<T: DeserializeOwned>(
        &mut self,
        uris: &Vec<Uri>,
        method: Method,
        bodies: &Vec<Option<Value>>,
    ) -> Result<Vec<(Option<T>, Option<anyhow::Error>)>, anyhow::Error> {
        json_http::request_batch(&mut self.handler, uris, method, bodies).await
    }

    pub async fn get<T: DeserializeOwned>(
        &mut self,
        uri: &Uri,
        params: Option<HashMap<String, String>>,
        headers: Option<Vec<(String, String)>>,
    ) -> Result<T, anyhow::Error> {
        let uri = json_http::build_uri_with_params(uri, params);
        json_http::request(&mut self.handler, &uri, Method::GET, None, headers).await
    }

    pub async fn post<T: DeserializeOwned>(
        &mut self,
        uri: &Uri,
        body: Option<Value>,
        headers: Option<Vec<(String, String)>>,
    ) -> Result<T, anyhow::Error> {
        json_http::request(&mut self.handler, &uri, Method::POST, body, headers).await
    }

    pub async fn put<T: DeserializeOwned>(
        &mut self,
        uri: &Uri,
        body: Option<Value>,
        headers: Option<Vec<(String, String)>>,
    ) -> Result<T, anyhow::Error> {
        json_http::request(&mut self.handler, &uri, Method::PUT, body, headers).await
    }

    pub async fn delete<T: DeserializeOwned>(
        &mut self,
        uri: &Uri,
        headers: Option<Vec<(String, String)>>,
    ) -> Result<T, anyhow::Error> {
        json_http::request(&mut self.handler, &uri, Method::DELETE, None, headers).await
    }
}
