use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{sleep, Instant};

pub struct RateLimiter {
    inner: Mutex<Inner>,
}

struct Inner {
    capacity: f64,
    tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl RateLimiter {
    /// `capacity` — max burst
    /// `refill_rate` — tokens per second (e.g. 0.333 for 1 per 3 seconds)
    pub fn new(capacity: u64, refill_rate: f64) -> Self {
        assert!(capacity > 0);
        assert!(refill_rate > 0.0);

        let now = Instant::now();

        Self {
            inner: Mutex::new(Inner {
                capacity: capacity as f64,
                tokens: capacity as f64,
                refill_rate,
                last_refill: now,
            }),
        }
    }

    pub async fn acquire(&self) {
        loop {
            let mut inner = self.inner.lock().await;
            let now = Instant::now();

            let elapsed = now.duration_since(inner.last_refill).as_secs_f64();
            inner.tokens = (inner.tokens + elapsed * inner.refill_rate).min(inner.capacity);
            inner.last_refill = now;

            if inner.tokens >= 1.0 {
                inner.tokens -= 1.0;
                return;
            }

            // Compute wait time for next token
            let needed = 1.0 - inner.tokens;
            let wait_secs = needed / inner.refill_rate;

            drop(inner);
            sleep(Duration::from_secs_f64(wait_secs)).await;
        }
    }
}
