use http_body_util::BodyExt;
use hyper::{
    body::{Body, Bytes},
    client::conn::http1,
    rt::{Read, Write},
    Request,
};
use hyper_util::rt::TokioExecutor;
use std::{error::Error, time::Duration};
use tokio::time::timeout;

pub struct HttpClient {}

impl HttpClient {
    pub async fn request<T, B>(stream: T, request: Request<B>) -> Result<Bytes, anyhow::Error>
    where
        T: Read + Write + Unpin + Send + 'static,
        B: Body + 'static + Unpin + Send,
        B::Data: Send,
        B::Error: Into<Box<dyn Error + Send + Sync>>,
    {
        let (mut sender, connection) = timeout(
            Duration::from_secs(5),
            http1::handshake(stream),
        )
        .await??;

        tokio::task::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("HTTP/2 connection error: {}", e);
            }
        });

        let res = sender.send_request(request).await?;

        Ok(res.collect().await?.to_bytes())
    }
}
