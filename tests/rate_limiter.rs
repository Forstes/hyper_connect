use hyper_connect::handlers::rate_limiter::RateLimiter;
use std::time::{Duration, Instant};

#[tokio::test]
async fn rate_is_respected_over_time() {
    let limiter = RateLimiter::new(1, 1.0);

    let start = Instant::now();

    limiter.acquire().await;
    limiter.acquire().await;
    limiter.acquire().await;

    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_secs(2));
}
