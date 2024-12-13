use hyper::{Method, Uri};
use hyper_connect::clients::http2::Http2Client;
use serde_json::Value;

#[tokio::test]
async fn test_http2() {
    let mut client = Http2Client::new();

    //let uri = Uri::from_static("https://api.binance.com/api/v3/ticker/price");
    // let response: Value = client.get(&uri, None).await.unwrap();
    // println!("Binance Server Time Response: {}", response);

    let uris: Vec<Uri> = vec![
        "https://api.binance.com/api/v3/ticker/price"
            .parse()
            .unwrap(),
        "https://api.binance.com/api/v3/time".parse().unwrap(),
        "https://api.binance.com/api/v3/exchangeInfo"
            .parse()
            .unwrap(),
    ];

    let results: Vec<(Option<Value>, Option<anyhow::Error>)> = client
        .request_many(&uris, Method::GET, &Vec::new())
        .await
        .unwrap();

    for (opt_value, opt_error) in results {
        match (opt_value, opt_error) {
            (Some(value), None) => {
                println!("Success: {:?}", value);
            }
            (None, Some(error)) => {
                println!("Error: {:?}", error);
            }
            (None, None) => {
                println!("No value and no error.");
            }
            (Some(_), Some(_)) => {
                println!("Unexpected: Both value and error are present.");
            }
        }
    }
}
