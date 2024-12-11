use hyper::Uri;
use hyper_connect::clients::http1::Http1Client;
use serde_json::Value;

#[tokio::test]
async fn test_http1() {
    let mut client = Http1Client::new();

    let uri = Uri::from_static("https://api.binance.com/api/v3/time");
    match client.get::<Value>(&uri).await {
        Ok(response) => {
            println!("Binance Server Time Response: {}", response);
        }
        Err(e) => {
            eprintln!("Request failed: {:?}", e);
        }
    }
}
