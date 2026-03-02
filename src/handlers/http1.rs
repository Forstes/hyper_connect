use crate::handlers::http1_conn_pool::Http1ConnPool;
#[cfg(feature = "rate_limit")]
use crate::handlers::rate_limiter::RateLimiter;
use crate::{connectors::types::Http1Connector, handlers::traits::HttpHandler};
use bytes::Bytes;
use http_body_util::{BodyExt, Either, Empty, Full};
use hyper::{Request, StatusCode, Uri};

pub struct Http1Handler<CN: Http1Connector> {
    conn_pool: Http1ConnPool<CN>,
    #[cfg(feature = "rate_limit")]
    rate_limiter: RateLimiter,
}

impl<CN: Http1Connector> Http1Handler<CN> {
    pub fn new(conn_pool: Http1ConnPool<CN>) -> Self {
        Self {
            conn_pool,
            #[cfg(feature = "rate_limit")]
            rate_limiter: RateLimiter::new(1000, 1.0),
        }
    }

    #[cfg(feature = "rate_limit")]
    pub fn new_with_rate_limit(conn_pool: Http1ConnPool<CN>, capacity: u64, refill_per_sec: f64) -> Self {
        Self {
            conn_pool,
            rate_limiter: RateLimiter::new(capacity, refill_per_sec),
        }
    }
}

impl<CN: Http1Connector + Send + Sync> HttpHandler for Http1Handler<CN> {
    async fn request(&self, uri: &Uri, request: Request<Either<Full<Bytes>, Empty<Bytes>>>) -> Result<(StatusCode, Bytes), anyhow::Error> {
        let mut conn = self.conn_pool.get_conn(uri).await?;

        #[cfg(feature = "rate_limit")]
        self.rate_limiter.acquire().await;

        let resp = conn.send_request(request).await?;
        let status = resp.status();
        let collected = resp.into_body().collect().await?;

        self.conn_pool.return_conn(uri, conn).await;

        Ok((status, collected.to_bytes()))
    }
}
