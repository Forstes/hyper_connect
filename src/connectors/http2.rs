use super::HttpConnector;
use crate::{connection::Http2Connection, utils::tls::create_tls_connector};
use hyper::{body::Body, client::conn::http2};
use hyper_util::rt::TokioIo;
use std::time::Duration;
use tokio::{net::TcpStream, time::timeout};

pub struct SimpleHttp2Connector {}

impl HttpConnector for SimpleHttp2Connector {
    type Connection<B>
        = Http2Connection<B>
    where
        B: Body + Unpin + Send + Sync + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>;

    async fn create_connection<B>(&self, uri: &hyper::Uri) -> Result<Self::Connection<B>, anyhow::Error>
    where
        B: Body + 'static + Unpin + Send + Sync,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let authority = uri.authority().unwrap();
        let socket_address = if !authority.as_str().contains(":") {
            // Add the default port 443 for HTTPS
            format!("{}:443", authority)
        } else {
            authority.as_str().to_string()
        };

        let tcp_stream = TcpStream::connect(socket_address).await?;
        let tls = create_tls_connector(true);
        let domain = tokio_rustls::rustls::pki_types::ServerName::try_from(uri.host().unwrap().to_string())?;
        let tls_stream = TokioIo::new(tls.connect(domain, tcp_stream).await?);
        let executor = hyper_util::rt::tokio::TokioExecutor::new();

        let (sender, connection) = timeout(Duration::from_secs(5), http2::handshake(executor, tls_stream)).await??;

        tokio::task::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("HTTP/2 connection error: {}", e);
            }
        });

        Ok(Http2Connection { conn: sender })
    }
}
