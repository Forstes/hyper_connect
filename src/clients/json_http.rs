use crate::handlers::http::HttpHandler;
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::{Method, Request, Uri};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub async fn request<T: DeserializeOwned>(
    handler: &mut HttpHandler<Either<Full<Bytes>, Empty<Bytes>>>,
    uri: &Uri,
    method: Method,
    body: Option<Value>,
) -> Result<T, anyhow::Error> {
    let body_data: Either<Full<Bytes>, Empty<Bytes>> = match body {
        Some(b) => Either::Left(Full::from(b.to_string())),
        None => Either::Right(Empty::<Bytes>::new()),
    };

    let mut builder = Request::builder()
        .method(&method)
        .uri(uri)
        .header("User-Agent", "RustClient/1.0")
        .header(hyper::header::HOST, uri.authority().unwrap().as_str());

    if method != Method::GET {
        builder = builder.header("Content-Type", "application/json")
    }

    let request = builder.body(body_data).expect("Failed to build request");
    println!("Sending request: {:?}", request);
    let uri = request.uri().clone();
    let (status, body) = handler.request(&uri, request).await?;

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
