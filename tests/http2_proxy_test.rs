use hyper::Uri;
use hyper_connect::clients::http2_proxy::Http2ProxyClient;
use serde_json::Value;

#[tokio::test]
async fn test_request() {
    let mut client = Http2ProxyClient::new(
        "198.23.239.134:6540".to_string(),
        "dxjhvrqm".to_string(),
        "1p487vhyxa6s".to_string(),
    );

    let uri = Uri::from_static("http://www.randomnumberapi.com/api/v1.0/random");

    client.get::<Value>(&uri, None, None).await.expect("Request failed");
}
