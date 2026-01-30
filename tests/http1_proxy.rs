use hyper_connect::clients::http1_proxy::*;
use serde_json::Value;
use std::env;

#[tokio::test]
async fn test_request() {
    dotenvy::dotenv().ok();

    let proxy_address = env::var("PROXY_ADDRESS").expect("PROXY_ADDRESS must be set");
    let proxy_username = env::var("PROXY_USERNAME").expect("PROXY_USERNAME must be set");
    let proxy_password = env::var("PROXY_PASSWORD").expect("PROXY_PASSWORD must be set");

    let client = Http1ProxyClient::new(proxy_address, proxy_username, proxy_password, 1);

    let resp = client
        .get("https://www.randomnumberapi.com/api/v1.0/random")
        .send()
        .await
        .unwrap()
        .to_json::<Value>()
        .unwrap();

    println!("{}", resp);
}
