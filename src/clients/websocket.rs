use crate::utils::tls::create_tls_connector;
use fastwebsockets::FragmentCollector;
use http_body_util::Empty;
use hyper::{
    body::Bytes,
    header::{CONNECTION, UPGRADE},
    upgrade::Upgraded,
    Request,
};
use hyper_util::rt::TokioIo;
use std::future::Future;
use tokio::{net::TcpStream, task};
use tokio_rustls::rustls::pki_types::ServerName;

pub use fastwebsockets::OpCode;

pub type WebSocket = FragmentCollector<TokioIo<Upgraded>>;

pub struct TextWebSocketStream {
    stream: WebSocket,
}

impl TextWebSocketStream {
    pub fn new(stream: WebSocket) -> Self {
        Self { stream }
    }

    pub async fn next(&mut self) -> anyhow::Result<Bytes> {
        loop {
            let frame = self.stream.read_frame().await?;

            match frame.opcode {
                OpCode::Text => return Ok(Bytes::copy_from_slice(&frame.payload)),
                OpCode::Close => anyhow::bail!("WebSocket stream closed"),
                _ => continue,
            }
        }
    }
}

pub async fn connect(host: &str, path: &str) -> anyhow::Result<WebSocket> {
    let tcp_stream = TcpStream::connect((host, 443)).await?;
    let domain = ServerName::try_from(host.to_owned())?;
    let tls_stream = create_tls_connector(false).connect(domain, tcp_stream).await?;
    let request = Request::builder()
        .method("GET")
        .uri(format!("wss://{host}{path}"))
        .header("Host", host)
        .header(UPGRADE, "websocket")
        .header(CONNECTION, "upgrade")
        .header("Sec-WebSocket-Key", fastwebsockets::handshake::generate_key())
        .header("Sec-WebSocket-Version", "13")
        .body(Empty::<Bytes>::new())?;
    let (stream, _) = fastwebsockets::handshake::client(&SpawnExecutor, request, tls_stream).await?;

    Ok(FragmentCollector::new(stream))
}

struct SpawnExecutor;

impl<Fut> hyper::rt::Executor<Fut> for SpawnExecutor
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    fn execute(&self, future: Fut) {
        task::spawn(future);
    }
}
