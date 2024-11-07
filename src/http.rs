use crate::tls::create_tls_connector;
use hyper::{
    body::Body,
    client::conn::http1::{self, SendRequest},
};
use hyper_util::rt::TokioIo;
use std::{error::Error, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    time::timeout,
};

pub async fn establish_http1_conn<T, B>(
    stream: T,
    domain: String,
) -> Result<SendRequest<B>, anyhow::Error>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    B: Body + 'static + Unpin + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
{
    let tls = create_tls_connector();
    let domain = tokio_rustls::rustls::pki_types::ServerName::try_from(domain)?;
    let tls_stream = TokioIo::new(tls.connect(domain, stream).await?);

    let (sender, connection) =
        timeout(Duration::from_secs(5), http1::handshake(tls_stream)).await??;

    tokio::task::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("HTTP/1 connection error: {}", e);
        }
    });

    Ok(sender)
}
