use hyper_connect::clients::http1_proxy::Http1ProxyClient;
use serde_json::Value;
use std::env;

#[tokio::test]
async fn test_request() {
    dotenvy::dotenv().ok();

    let proxy_address = env::var("PROXY_ADDRESS").expect("PROXY_ADDRESS must be set");
    let proxy_username = env::var("PROXY_USERNAME").expect("PROXY_USERNAME must be set");
    let proxy_password = env::var("PROXY_PASSWORD").expect("PROXY_PASSWORD must be set");

    let mut client = Http1ProxyClient::new(proxy_address, proxy_username, proxy_password);

    let resp = client
        .get::<Value>("https://www.randomnumberapi.com/api/v1.0/random", None, None)
        .await
        .expect("Request failed");

    println!("{}", resp);
}
