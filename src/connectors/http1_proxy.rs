use crate::{
    connection::Http1Connection,
    connectors::HttpConnector,
    utils::{proxy_tunnel::create_proxy_tunnel, tls::create_tls_connector},
};
use hyper::{body::Body, client::conn::http1};
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

impl HttpConnector for ProxyHttp1Connector {
    type Connection<B>
        = Http1Connection<B>
    where
        B: Body + Unpin + Send + Sync + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>;

    async fn create_connection<B>(&self, target_uri: &hyper::Uri) -> Result<Self::Connection<B>, anyhow::Error>
    where
        B: Body + 'static + Unpin + Send + Sync,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
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

        Ok(Http1Connection { conn: sender })
    }
}
