use hyper_connect::clients::http2::*;
use serde_json::Value;

#[tokio::test]
async fn test_request() {
    let client: Http2Client = Http2Client::new(1);

    let resp = client
        .get("https://api1.binance.com/api/v3/time")
        .headers(vec![("Yo", "HoHO".to_string())])
        .send()
        .await
        .unwrap()
        .to_json::<Value>()
        .unwrap();

    println!("{resp}");
}
