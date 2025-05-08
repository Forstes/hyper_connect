use hyper_connect::clients::http2_new::Http2NewClient;
use serde_json::Value;

#[tokio::test]
async fn test_get() {
    let client = Http2NewClient::new(1);

    let resp = client
        .get("https://api1.binance.com/api/v3/time")
        .headers(vec![("Yo", "HoHO")])
        .send()
        .await
        .unwrap()
        .to_json::<Value>()
        .unwrap();

    println!("{resp}");
}
