use super::json_http;
use crate::{
    connectors::{enums::connector::ConnectorEnum, http1_proxy::ProxyHttp1Connector},
    handlers::http::HttpHandler,
};
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::{Method, Uri};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct Http1ProxyClient {
    handler: HttpHandler<Either<Full<Bytes>, Empty<Bytes>>>,
}

impl Http1ProxyClient {
    pub fn new(proxy_address: String, username: String, password: String) -> Self {
        let connector = ProxyHttp1Connector::new(proxy_address, username, password);
        let handler = HttpHandler::new(ConnectorEnum::Http1Proxy(connector));
        Self { handler }
    }

    pub async fn request_many<T: DeserializeOwned>(
        &mut self,
        uris: &Vec<Uri>,
        method: Method,
        bodies: &Vec<Option<Value>>,
    ) -> Result<Vec<(Option<T>, Option<anyhow::Error>)>, anyhow::Error> {
        json_http::request_batch(&mut self.handler, uris, method, bodies, true).await
    }

    pub async fn get<T: DeserializeOwned>(
        &mut self,
        uri: &str,
        params: Option<Value>,
        headers: Option<Vec<(String, String)>>,
    ) -> Result<T, anyhow::Error> {
        let uri = json_http::build_uri_with_params(uri, params)?;
        json_http::request::<Value, T>(&mut self.handler, &uri, Method::GET, None, headers, true).await
    }

    pub async fn post<T: DeserializeOwned>(
        &mut self,
        uri: &str,
        body: Option<Value>,
        headers: Option<Vec<(String, String)>>,
    ) -> Result<T, anyhow::Error> {
        json_http::request(&mut self.handler, uri, Method::POST, body, headers, true).await
    }

    pub async fn put<T: DeserializeOwned>(
        &mut self,
        uri: &str,
        body: Option<Value>,
        headers: Option<Vec<(String, String)>>,
    ) -> Result<T, anyhow::Error> {
        json_http::request(&mut self.handler, uri, Method::PUT, body, headers, true).await
    }

    pub async fn delete<T: DeserializeOwned>(
        &mut self,
        uri: &str,
        headers: Option<Vec<(String, String)>>,
    ) -> Result<T, anyhow::Error> {
        json_http::request::<Value, T>(&mut self.handler, uri, Method::DELETE, None, headers, true).await
    }
}
