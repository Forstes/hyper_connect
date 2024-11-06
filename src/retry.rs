use std::future::Future;
use tokio::time::{sleep, Duration};

pub async fn retry_on_error<F, T, E, CheckFn>(
    operation: impl Fn() -> F,
    should_retry: CheckFn,
    max_retries: u32,
    base_delay: Duration,
) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
    CheckFn: Fn(&E) -> bool,
{
    let mut attempts = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_retries && should_retry(&e) => {
                attempts += 1;
                let delay = base_delay * 2_u32.pow(attempts - 1); // Exponential backoff
                eprintln!("Operation failed: {:?}. Retrying in {:?}", e, delay);
                sleep(delay).await;
            }
            Err(e) => return Err(e), // Stop retrying if max retries reached or error is non-retryable
        }
    }
}
