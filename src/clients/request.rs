use crate::handlers::traits::HttpHandler;
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::{Method, Uri};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::de::SliceRead;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct RetryPolicy {
    max_attempts: usize,
    initial_backoff: Duration,
}

impl RetryPolicy {
    pub fn new(max_attempts: usize, initial_backoff: Duration) -> Self {
        assert!(max_attempts > 0, "max_attempts must be greater than zero");

        Self {
            max_attempts,
            initial_backoff,
        }
    }

    fn delay_for_retry(&self, failed_attempt: usize) -> Duration {
        let multiplier = 1_u32.checked_shl(failed_attempt.try_into().unwrap_or(u32::MAX)).unwrap_or(u32::MAX);
        self.initial_backoff.saturating_mul(multiplier)
    }
}

pub struct Request<'a, H: HttpHandler> {
    pub(crate) handler: &'a H,
    pub(crate) uri: &'a str,
    pub(crate) method: Method,
    pub(crate) query: String,
    pub(crate) body: Option<Vec<u8>>,
    pub(crate) headers: Vec<(&'a str, String)>,
    pub(crate) include_host_header: bool,
    pub(crate) retry_policy: Option<RetryPolicy>,
}

impl<'a, H: HttpHandler> Request<'a, H> {
    pub fn query<S: Serialize>(mut self, v: &S) -> Result<Self, serde_urlencoded::ser::Error> {
        self.query = serde_urlencoded::to_string(v)?;
        Ok(self)
    }

    pub fn json<S: Serialize>(mut self, v: &S) -> Result<Self, serde_json::Error> {
        self.body = Some(serde_json::to_vec(v)?);
        self.headers.push(("Content-Type", "application/json".to_string()));
        Ok(self)
    }

    pub fn form<S: Serialize>(mut self, v: &S) -> Result<Self, serde_urlencoded::ser::Error> {
        self.body = Some(serde_urlencoded::to_string(v)?.into_bytes());
        self.headers.push(("Content-Type", "application/x-www-form-urlencoded".to_string()));
        Ok(self)
    }

    pub fn headers(mut self, mut headers: Vec<(&'a str, String)>) -> Self {
        self.headers.append(&mut headers);
        self
    }

    pub fn retry(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = Some(retry_policy);
        self
    }

    pub async fn send(self) -> Result<ResponseData, anyhow::Error> {
        let uri = if self.query.is_empty() {
            Uri::try_from(self.uri)?
        } else {
            let mut uri_buf = String::with_capacity(self.uri.len() + 1 + self.query.len());
            uri_buf.push_str(self.uri);
            uri_buf.push('?');
            uri_buf.push_str(&self.query);
            Uri::try_from(uri_buf)?
        };

        let retry_policy = self
            .retry_policy
            .filter(|_| self.body.is_none() && (self.method == Method::GET || self.method == Method::HEAD));
        let max_attempts = retry_policy.as_ref().map_or(1, |policy| policy.max_attempts);

        for attempt in 0..max_attempts {
            let mut builder = hyper::Request::builder()
                .method(&self.method)
                .uri(&uri)
                .header("User-Agent", "RustClient/1.0");

            if self.include_host_header {
                builder = builder.header("HOST", uri.host().unwrap_or_default());
            }

            for (key, value) in &self.headers {
                builder = builder.header(*key, value);
            }

            let body_data = match &self.body {
                Some(body) => Either::Left(Full::from(body.clone())),
                None => Either::Right(Empty::<Bytes>::new()),
            };

            match self.handler.request(&uri, builder.body(body_data)?).await {
                Ok((status, body)) if status.is_success() => return Ok(ResponseData { data: body }),
                Ok((status, body)) => {
                    return Err(anyhow::anyhow!(
                        "Request failed with status {}: {}",
                        status,
                        String::from_utf8_lossy(&body)
                    ));
                }
                Err(_) if attempt + 1 < max_attempts => {
                    tokio::time::sleep(
                        retry_policy
                            .as_ref()
                            .expect("retry policy must exist when retrying")
                            .delay_for_retry(attempt),
                    )
                    .await;
                }
                Err(error) => return Err(error),
            }
        }

        unreachable!("retry loop must return a response or error")
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FlakyHandler {
        attempts: AtomicUsize,
    }

    impl HttpHandler for FlakyHandler {
        async fn request(&self, _: &Uri, _: hyper::Request<Either<Full<Bytes>, Empty<Bytes>>>) -> Result<(hyper::StatusCode, Bytes), anyhow::Error> {
            if self.attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                anyhow::bail!("deadline has elapsed");
            }

            Ok((hyper::StatusCode::OK, Bytes::from_static(b"{}")))
        }
    }

    #[tokio::test]
    async fn retries_safe_requests_after_transport_errors() {
        let handler = FlakyHandler {
            attempts: AtomicUsize::new(0),
        };
        let request = Request {
            handler: &handler,
            uri: "https://example.com",
            method: Method::GET,
            query: String::new(),
            body: None,
            headers: Vec::new(),
            include_host_header: false,
            retry_policy: Some(RetryPolicy::new(2, Duration::ZERO)),
        };

        request.send().await.unwrap();

        assert_eq!(handler.attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn does_not_retry_unsafe_requests() {
        let handler = FlakyHandler {
            attempts: AtomicUsize::new(0),
        };
        let request = Request {
            handler: &handler,
            uri: "https://example.com",
            method: Method::POST,
            query: String::new(),
            body: None,
            headers: Vec::new(),
            include_host_header: false,
            retry_policy: Some(RetryPolicy::new(2, Duration::ZERO)),
        };

        assert!(request.send().await.is_err());
        assert_eq!(handler.attempts.load(Ordering::SeqCst), 1);
    }
}
