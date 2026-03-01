use crate::{
    connectors::types::{Http1Connection, Http1Connector},
    utils::{proxy_tunnel::create_proxy_tunnel, tls::create_tls_connector},
};
use hyper::client::conn::http1;
use hyper_util::rt::TokioIo;
use std::time::Duration;
use tokio::time::timeout;

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
}

impl Http1Connector for ProxyHttp1Connector {
    async fn create_connection(&self, target_uri: &hyper::Uri) -> Result<Http1Connection, anyhow::Error> {
        let tcp_stream = create_proxy_tunnel(target_uri, &self.proxy_address, &self.username, &self.password, "1.1").await?;
        let tls = create_tls_connector(false);
        let domain = tokio_rustls::rustls::pki_types::ServerName::try_from(target_uri.host().unwrap().to_string())?;
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
