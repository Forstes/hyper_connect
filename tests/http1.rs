use hyper_connect::clients::http1::*;
use serde_json::Value;

#[tokio::test]
async fn test_request() {
    let client = Http1Client::new(1);

    let resp = client
        .get("https://api1.binance.com/api/v3/time")
        .send()
        .await
        .unwrap()
        .to_json::<Value>()
        .unwrap();

    println!("{}", resp);
}
