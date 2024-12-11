use crate::{
    connectors::{enums::connector::ConnectorEnum, http2::SimpleHttp2Connector},
    handlers::http::HttpHandler,
};
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::{Method, Request, Uri};
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

    pub async fn get<T: DeserializeOwned>(&mut self, uri: &Uri) -> Result<T, anyhow::Error> {
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("User-Agent", "RustHttp2Client/1.0")
            .body(Either::Right(Empty::<Bytes>::new()))
            .expect("Failed to build GET request");
        self.handle_request(request).await
    }

    pub async fn post<T: DeserializeOwned>(
        &mut self,
        uri: &Uri,
        body: Value,
    ) -> Result<T, anyhow::Error> {
        let request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Content-Type", "application/json")
            .header("User-Agent", "RustHttp2Client/1.0")
            .body(Either::Left(Full::from(body.to_string())))
            .expect("Failed to build POST request");
        self.handle_request(request).await
    }

    pub async fn put<T: DeserializeOwned>(
        &mut self,
        uri: &Uri,
        body: Value,
    ) -> Result<T, anyhow::Error> {
        let request = Request::builder()
            .method(Method::PUT)
            .uri(uri)
            .header("Content-Type", "application/json")
            .header("User-Agent", "RustHttp2Client/1.0")
            .body(Either::Left(Full::from(body.to_string())))
            .expect("Failed to build PUT request");
        self.handle_request(request).await
    }

    pub async fn delete<T: DeserializeOwned>(&mut self, uri: &Uri) -> Result<T, anyhow::Error> {
        let request = Request::builder()
            .method(Method::DELETE)
            .uri(uri)
            .header("User-Agent", "RustHttp2Client/1.0")
            .body(Either::Right(Empty::<Bytes>::new()))
            .expect("Failed to build DELETE request");
        self.handle_request(request).await
    }

    async fn handle_request<T: DeserializeOwned>(
        &mut self,
        request: Request<Either<Full<Bytes>, Empty<Bytes>>>,
    ) -> Result<T, anyhow::Error> {
        let uri = request.uri().clone();
        let (status, body) = self.handler.request(&uri, request).await?;
        if status.is_success() {
            let parsed: T = serde_json::from_slice(&body)?;
            Ok(parsed)
        } else {
            Err(anyhow::anyhow!(
                "Request failed with status {}: {}",
                status,
                String::from_utf8_lossy(&body)
            ))
        }
    }
}
