use crate::handlers::http2_conn_pool::Http2ConnPool;
#[cfg(feature = "rate_limit")]
use crate::handlers::rate_limiter::RateLimiter;
use crate::{connectors::types::Http2Connector, handlers::traits::HttpHandler};
use bytes::Bytes;
use http_body_util::{BodyExt, Either, Empty, Full};
use hyper::{Request, StatusCode, Uri};

pub struct Http2Handler<CN: Http2Connector> {
    conn_pool: Http2ConnPool<CN>,
    #[cfg(feature = "rate_limit")]
    rate_limiter: RateLimiter,
}

impl<CN: Http2Connector> Http2Handler<CN> {
    pub fn new(conn_pool: Http2ConnPool<CN>) -> Self {
        Self {
            conn_pool,
            #[cfg(feature = "rate_limit")]
            rate_limiter: RateLimiter::new(1000, 1),
        }
    }

    #[cfg(feature = "rate_limit")]
    pub fn new_with_rate_limit(conn_pool: Http2ConnPool<CN>, capacity: u64, refill_per_sec: u64) -> Self {
        Self {
            conn_pool,
            rate_limiter: RateLimiter::new(capacity, refill_per_sec),
        }
    }
}

impl<CN: Http2Connector + Send + Sync> HttpHandler for Http2Handler<CN> {
    async fn request(&self, uri: &Uri, request: Request<Either<Full<Bytes>, Empty<Bytes>>>) -> Result<(StatusCode, Bytes), anyhow::Error> {
        let mut conn = self.conn_pool.get_conn(uri).await?;

        #[cfg(feature = "rate_limit")]
        self.rate_limiter.acquire().await;

        let resp = conn.send_request(request).await?;
        let status = resp.status();
        let collected = resp.into_body().collect().await?;

        Ok((status, collected.to_bytes()))
    }
}
