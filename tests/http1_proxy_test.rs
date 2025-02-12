use hyper::Uri;
use hyper_connect::clients::http1_proxy::Http1ProxyClient;
use serde_json::Value;

#[tokio::test]
async fn test_request() {
    let mut client = Http1ProxyClient::new(
        "198.23.239.134:6540".to_string(),
        "dxjhvrqm".to_string(),
        "1p487vhyxa6s".to_string(),
    );

    let uri = Uri::from_static("http://www.randomnumberapi.com/api/v1.0/random");

    let resp = client.get::<Value>(&uri, None, None).await.expect("Request failed");

    println!("{}", resp);
}
