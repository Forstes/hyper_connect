use hyper_connect::clients::http2::*;
use serde_json::Value;
use std::time::Instant;

const TEST_URL: &str = "https://api1.binance.com/api/v3/time";

#[tokio::test]
async fn test_request() {
    let client: Http2Client = Http2Client::new();

    let resp = client
        .get(TEST_URL)
        .headers(vec![("Yo", "HoHO".to_string())])
        .send()
        .await
        .unwrap()
        .to_json::<Value>()
        .unwrap();

    println!("{resp}");
}

#[tokio::test]
async fn rate_limiter_enforces_delay() {
    // Allow 2 immediate requests
    // Refill 1 request per second
    let client = Http2Client::new_with_rate_limit(2, 1.0);

    let start = Instant::now();

    // Fire 4 requests in parallel
    let r1 = client.get(TEST_URL).send();
    let r2 = client.get(TEST_URL).send();
    let r3 = client.get(TEST_URL).send();
    let r4 = client.get(TEST_URL).send();

    let _ = tokio::join!(r1, r2, r3, r4);

    let elapsed = start.elapsed();

    // First 2 are instant (burst)
    // Next 2 should take at least ~2 seconds total (1 token per second refill)
    assert!(
        elapsed.as_secs_f64() >= 2.0,
        "Rate limiter did not delay properly: elapsed = {:?}",
        elapsed
    );
}
