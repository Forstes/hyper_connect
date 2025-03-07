use crate::handlers::http::HttpHandler;
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::{Method, Request, Uri};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub async fn request<T: DeserializeOwned>(
    handler: &mut HttpHandler<Either<Full<Bytes>, Empty<Bytes>>>,
    uri: &str,
    method: Method,
    body: Option<Value>,
    headers: Option<Vec<(String, String)>>,
) -> Result<T, anyhow::Error> {
    let body_data: Either<Full<Bytes>, Empty<Bytes>> = match body {
        Some(b) => Either::Left(Full::from(b.to_string())),
        None => Either::Right(Empty::<Bytes>::new()),
    };

    let mut builder = Request::builder()
        .method(&method)
        .uri(uri)
        .header("User-Agent", "RustClient/1.0");

    if let Some(header_list) = headers {
        for (key, value) in header_list {
            builder = builder.header(key, value);
        }
    }

    if method != Method::GET {
        builder = builder.header("Content-Type", "application/json")
    }

    let request = builder.body(body_data).expect("Failed to build request");
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

pub async fn request_batch<T: DeserializeOwned>(
    handler: &mut HttpHandler<Either<Full<Bytes>, Empty<Bytes>>>,
    uris: &Vec<Uri>,
    method: Method,
    bodies: &Vec<Option<Value>>,
) -> Result<Vec<(Option<T>, Option<anyhow::Error>)>, anyhow::Error> {
    let mut requests = Vec::new();

    for (i, uri) in uris.iter().enumerate() {
        let body_data: Either<Full<Bytes>, Empty<Bytes>> = match bodies.get(i) {
            Some(Some(b)) => Either::Left(Full::from(b.to_string())),
            _ => Either::Right(Empty::<Bytes>::new()),
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
        requests.push(request);
    }

    let responses = handler.request_many(&uris[0], requests).await?;

    let mut results = Vec::with_capacity(responses.len());
    for (status, body) in responses {
        if status.is_success() {
            match serde_json::from_slice::<T>(&body) {
                Ok(parsed) => {
                    results.push((Some(parsed), None));
                }
                Err(e) => {
                    results.push((None, Some(anyhow::anyhow!("Failed to deserialize: {}", e))));
                }
            }
        } else {
            let error_message = anyhow::anyhow!("Request failed with status {}: {}", status, String::from_utf8_lossy(&body));
            results.push((None, Some(error_message)));
        }
    }

    Ok(results)
}

pub fn build_uri_with_params(base_uri: &str, params: Option<Value>) -> anyhow::Result<String> {
    if let Some(params) = params {
        if let Value::Object(ref obj) = params {
            let query = obj
                .iter()
                .map(|(key, val)| {
                    let value_str = match val {
                        Value::String(s) => s.clone(),
                        _ => val.to_string(),
                    };

                    format!(
                        "{}={}",
                        utf8_percent_encode(key, NON_ALPHANUMERIC),
                        utf8_percent_encode(&value_str, NON_ALPHANUMERIC)
                    )
                })
                .collect::<Vec<String>>()
                .join("&");

            return Ok(format!("{}?{}", base_uri, query));
        }

        return Err(anyhow::anyhow!("params must be an object"));
    }

    Ok(base_uri.to_string())
}
