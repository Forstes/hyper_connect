use super::enums::SendRequestEnum;
use crate::tls::create_tls_connector;
use hyper::{body::Body, client::conn::http2, Uri};
use hyper_util::rt::TokioIo;
use std::{error::Error, time::Duration};
use tokio::{net::TcpStream, time::timeout};

pub trait HttpConnector<B>
where
    B: Body + 'static + Unpin + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
{
    async fn create_connection(&self, uri: &Uri) -> Result<SendRequestEnum<B>, anyhow::Error>;
}

pub struct SimpleHttp2Connector {}

impl<B> HttpConnector<B> for SimpleHttp2Connector
where
    B: Body + 'static + Unpin + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    async fn create_connection(&self, uri: &Uri) -> Result<SendRequestEnum<B>, anyhow::Error> {
        let tcp_stream = TcpStream::connect(uri.authority().unwrap().as_str()).await?;
        let tls = create_tls_connector();
        let domain =
            tokio_rustls::rustls::pki_types::ServerName::try_from(uri.host().unwrap().to_string())?;
        let tls_stream = TokioIo::new(tls.connect(domain, tcp_stream).await?);
        let executor = hyper_util::rt::tokio::TokioExecutor::new();

        let (sender, connection) = timeout(
            Duration::from_secs(5),
            http2::handshake(executor, tls_stream),
        )
        .await??;

        tokio::task::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("HTTP/2 connection error: {}", e);
            }
        });

        Ok(SendRequestEnum::Http2(sender))
    }
}
