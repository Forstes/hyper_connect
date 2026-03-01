use crate::{
    connectors::types::{Http1Connection, Http1Connector},
    utils::tls::create_tls_connector,
};
use hyper::client::conn::http1;
use hyper_util::rt::TokioIo;
use std::time::Duration;
use tokio::{net::TcpStream, time::timeout};

pub struct SimpleHttp1Connector {}

impl Http1Connector for SimpleHttp1Connector {
    async fn create_connection(&self, uri: &hyper::Uri) -> Result<Http1Connection, anyhow::Error> {
        let authority = uri.authority().unwrap();
        let socket_address = if !authority.as_str().contains(":") {
            // Add the default port 443 for HTTPS
            format!("{}:443", authority)
        } else {
            authority.as_str().to_string()
        };

        let tcp_stream = TcpStream::connect(socket_address).await?;
        let tls = create_tls_connector(false);
        let domain = tokio_rustls::rustls::pki_types::ServerName::try_from(uri.host().unwrap().to_string())?;
        let tls_stream = TokioIo::new(tls.connect(domain, tcp_stream).await?);

        let (sender, connection) = timeout(Duration::from_secs(5), http1::handshake(tls_stream)).await??;

        tokio::task::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("HTTP/1 connection error: {}", e);
            }
        });

        Ok(sender)
    }
}
