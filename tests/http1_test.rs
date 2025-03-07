use hyper_connect::clients::http1::Http1Client;
use serde_json::Value;

#[tokio::test]
async fn test_http1() {
    let mut client = Http1Client::new();
    let params = serde_json::json!({"symbol": "BNBUSDT"});

    match client
        .get::<Value>("https://api.binance.com/api/v3/ticker/price", Some(params))
        .await
    {
        Ok(response) => {
            println!("{}", response);
        }
        Err(e) => {
            eprintln!("Request failed: {:?}", e);
        }
    }
}
