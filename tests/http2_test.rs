use hyper::Uri;
use hyper_connect::clients::http2::Http2Client;
use serde_json::Value;

#[tokio::test]
async fn test_http2() {
    let mut client = Http2Client::new();

    let uri = Uri::from_static("https://api.binance.com/api/v3/ticker/price");
    //println!("{}", uri.authority().unwrap().as_str())
    let response: Value = client.get(&uri).await.unwrap();

    println!("Binance Server Time Response: {}", response);
}
