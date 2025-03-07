use super::json_http;
use crate::{
    connectors::{enums::connector::ConnectorEnum, http1::SimpleHttp1Connector},
    handlers::http::HttpHandler,
};
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::Method;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct Http1Client {
    handler: HttpHandler<Either<Full<Bytes>, Empty<Bytes>>>,
}

impl Http1Client {
    pub fn new() -> Self {
        let connector = SimpleHttp1Connector {};
        let handler = HttpHandler::new(ConnectorEnum::Http1(connector));
        Self { handler }
    }

    pub async fn get<T: DeserializeOwned>(&mut self, uri: &str, params: Option<Value>) -> Result<T, anyhow::Error> {
        let uri = json_http::build_uri_with_params(uri, params)?;
        json_http::request(&mut self.handler, &uri, Method::GET, None, None).await
    }
}
