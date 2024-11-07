use http_body_util::Empty;
use hyper::{body::Bytes, Request, Uri};
use hyper_connect::proxy::HttpProxyClient;
use std::env;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenvy::dotenv().ok();

    let proxy_address = env::var("PROXY_ADDRESS").expect("Proxy address not found");
    let proxy_username = env::var("PROXY_USERNAME").expect("Proxy username not found");
    let proxy_password = env::var("PROXY_PASSWORD").expect("Proxy password not found");

    let mut client = HttpProxyClient::new(proxy_address, proxy_username, proxy_password);

    let target = Uri::builder()
        .scheme("https")
        .authority("google.com")
        .path_and_query("/")
        .build()
        .unwrap();

    let request = Request::builder()
        .uri(target.authority().unwrap().as_str())
        .body(Empty::<Bytes>::new())
        .unwrap();

    match client.request(&target, request).await {
        Ok((s, b)) => println!("{}: {}", s, b.len()),
        Err(e) => println!("{}", e),
    }
}
