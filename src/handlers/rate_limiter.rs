use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct RateLimiter {
    capacity: u64,
    tokens: AtomicU64,
    refill_interval_ns: u64,
    last_refill_ns: AtomicU64,
    start: Instant,
}

impl RateLimiter {
    pub fn new(capacity: u64, refill_per_sec: u64) -> Self {
        assert!(capacity > 0);
        assert!(refill_per_sec > 0);

        Self {
            capacity,
            tokens: AtomicU64::new(capacity),
            refill_interval_ns: 1_000_000_000 / refill_per_sec,
            last_refill_ns: AtomicU64::new(0),
            start: Instant::now(),
        }
    }

    #[inline]
    fn now_ns(&self) -> u64 {
        self.start.elapsed().as_nanos() as u64
    }

    #[inline]
    fn refill(&self, now: u64) {
        let last = self.last_refill_ns.load(Ordering::Relaxed);
        let elapsed = now.saturating_sub(last);

        if elapsed < self.refill_interval_ns {
            return;
        }

        let tokens_to_add = elapsed / self.refill_interval_ns;
        if tokens_to_add == 0 {
            return;
        }

        let new_last = last + tokens_to_add * self.refill_interval_ns;

        if self
            .last_refill_ns
            .compare_exchange(last, new_last, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            let prev = self.tokens.fetch_add(tokens_to_add, Ordering::AcqRel);
            if prev + tokens_to_add > self.capacity {
                self.tokens.store(self.capacity, Ordering::Release);
            }
        }
    }

    pub async fn acquire(&self) {
        loop {
            let now = self.now_ns();

            self.refill(now);

            let current = self.tokens.load(Ordering::Acquire);

            if current > 0 {
                if self
                    .tokens
                    .compare_exchange(current, current - 1, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
                {
                    return;
                }
                continue;
            }

            let last = self.last_refill_ns.load(Ordering::Acquire);
            let next_refill = last + self.refill_interval_ns;
            let wait_ns = next_refill.saturating_sub(now);

            sleep(Duration::from_nanos(wait_ns.max(1))).await;
        }
    }
}
