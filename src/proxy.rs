use base64::engine::general_purpose;
use base64::Engine;
use core::str;
use http_body_util::Empty;
use hyper::body::Bytes;
use hyper::{Request, Uri};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

use crate::http::HttpClient;
use crate::tls::create_tls_connector;

pub struct HttpProxyClient {
    address: String,
    username: String,
    password: String,
}

impl<'a> HttpProxyClient {
    pub fn new(address: String, username: String, password: String) -> Self {
        HttpProxyClient {
            address,
            username,
            password,
        }
    }

    pub async fn get(&self, uri: &'a Uri) -> Result<(), anyhow::Error> {
        let upstream_request = Request::builder()
            .uri(uri.authority().unwrap().as_str())
            .header("user-agent", "hyper-client-http2")
            .body(Empty::<Bytes>::new())?;

        let stream = self.create_tunnel(uri).await?;
        let resp_body = HttpClient::request(stream, upstream_request).await?;
        println!("{}", String::from_utf8_lossy(&resp_body));

        Ok(())
    }

    async fn create_tunnel(
        &self,
        uri: &Uri,
    ) -> Result<TokioIo<TlsStream<TcpStream>>, anyhow::Error> {
        let tcp_stream = TcpStream::connect(&self.address).await?;

        let host = uri.host().unwrap().to_string();
        let port = uri.port().map_or(443, |p| p.as_u16());
        let encoded_credentials =
            general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password));

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
        if !response_str.contains("200") {
            return Err(anyhow::anyhow!(
                "Failed to establish tunnel: {}",
                response_str
            ));
        }

        loop {
            let mut leftover = [0; 512];
            match tcp_stream.try_read(&mut leftover) {
                Ok(0) => break,    // No more data to read
                Ok(_) => continue, // Drain the buffer
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e.into()),
            }
        }

        // Upgrade to TLS after CONNECT tunnel is established
        let tls = create_tls_connector();
        let domain = tokio_rustls::rustls::pki_types::ServerName::try_from(host)?;
        let tls_stream = tls.connect(domain, tcp_stream).await?;
        return Ok(TokioIo::new(tls_stream));
    }
}
