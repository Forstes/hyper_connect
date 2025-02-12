use super::enums::send_request::SendRequestEnum;
use crate::tls::create_tls_connector;
use base64::{engine::general_purpose, Engine};
use core::str;
use hyper::{body::Body, client::conn::http1, Uri};
use hyper_util::rt::TokioIo;
use std::time::Duration;
use tokio::{net::TcpStream, time::timeout};

pub struct ProxyHttp1Connector {
    pub proxy_address: String,
    pub username: String,
    pub password: String,
}

impl ProxyHttp1Connector {
    pub fn new(proxy_address: String, username: String, password: String) -> Self {
        Self {
            proxy_address,
            username,
            password,
        }
    }

    pub async fn create_connection<B>(
        &self,
        target_uri: &Uri,
        proxy_address: &String,
        username: &String,
        password: &String,
    ) -> Result<SendRequestEnum<B>, anyhow::Error>
    where
        B: Body + 'static + Unpin + Send,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let tcp_stream = self
            .create_proxy_tunnel(target_uri, proxy_address, username, password)
            .await?;
        let tls = create_tls_connector(false);
        let domain = tokio_rustls::rustls::pki_types::ServerName::try_from(target_uri.host().unwrap().to_string())?;
        let tls_stream = TokioIo::new(tls.connect(domain, tcp_stream).await?);

        let (sender, connection) = timeout(Duration::from_secs(5), http1::handshake(tls_stream)).await??;

        tokio::task::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("HTTP/1 connection error: {}", e);
            }
        });

        Ok(SendRequestEnum::Http1(sender))
    }

    async fn create_proxy_tunnel(
        &self,
        target_uri: &Uri,
        proxy_address: &String,
        username: &String,
        password: &String,
    ) -> Result<TcpStream, anyhow::Error> {
        let tcp_stream = TcpStream::connect(proxy_address).await?;

        let host = target_uri.host().unwrap().to_string();
        let port = target_uri.port().map_or(443, |p| p.as_u16());
        let encoded_credentials = general_purpose::STANDARD.encode(format!("{}:{}", username, password));

        let connect_request = format!(
            "CONNECT {0}:{1} HTTP/1.1\r\n\
             Host: {0}:{1}\r\n\
             Proxy-Authorization: Basic {2}\r\n\
             \r\n",
            &host, port, encoded_credentials
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
}
