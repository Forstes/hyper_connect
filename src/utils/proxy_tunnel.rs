use base64::{engine::general_purpose, Engine};
use core::str;
use hyper::Uri;
use tokio::net::TcpStream;

pub async fn create_proxy_tunnel(
    target_uri: &Uri,
    proxy_address: &String,
    username: &String,
    password: &String,
    protocol_version: &str,
) -> Result<TcpStream, anyhow::Error> {
    let tcp_stream = TcpStream::connect(proxy_address).await?;

    let host = target_uri.host().unwrap().to_string();
    let port = target_uri.port().map_or(443, |p| p.as_u16());
    let encoded_credentials = general_purpose::STANDARD.encode(format!("{}:{}", username, password));

    let connect_request = format!(
        "CONNECT {0}:{1} HTTP/{3}\r\n\
         Host: {0}:{1}\r\n\
         Proxy-Authorization: Basic {2}\r\n\
         \r\n",
        &host, port, encoded_credentials, protocol_version
    );

    tcp_stream.writable().await?;
    tcp_stream.try_write(connect_request.as_bytes())?;

    let mut response = vec![0; 1024];
    tcp_stream.readable().await?;
    let n = tcp_stream.try_read(&mut response)?;

    let response_str = str::from_utf8(&response[..n])?;
    if !response_str.contains("200 OK") {
        return Err(anyhow::anyhow!("Failed to establish tunnel: {}", response_str));
    }

    Ok(tcp_stream)
}
