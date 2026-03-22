use crate::{
    connectors::types::{Http2Connection, Http2Connector, Http2Sender},
    utils::{proxy_tunnel::create_proxy_tunnel, tls::create_tls_connector},
};
use hyper::client::conn::http2;
use hyper_util::rt::TokioIo;
use std::time::Duration;
use tokio::time::timeout;

pub struct ProxyHttp2Connector {
    pub proxy_address: String,
    pub username: String,
    pub password: String,
}

impl ProxyHttp2Connector {
    pub fn new(proxy_address: String, username: String, password: String) -> Self {
        Self {
            proxy_address,
            username,
            password,
        }
    }
}

impl Http2Connector for ProxyHttp2Connector {
    async fn create_connection(&self, target_uri: &hyper::Uri) -> Result<(Http2Sender, Http2Connection), anyhow::Error> {
        let tcp_stream = create_proxy_tunnel(target_uri, &self.proxy_address, &self.username, &self.password, "2.0").await?;
        let tls = create_tls_connector(true);
        let domain = tokio_rustls::rustls::pki_types::ServerName::try_from(target_uri.host().unwrap().to_string())?;
        let tls_stream = TokioIo::new(tls.connect(domain, tcp_stream).await?);
        let executor = hyper_util::rt::tokio::TokioExecutor::new();

        let (sender, connection) = timeout(Duration::from_secs(5), http2::handshake(executor, tls_stream)).await??;

        Ok((sender, connection))
    }
}
