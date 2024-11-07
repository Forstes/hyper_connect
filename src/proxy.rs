use crate::http::establish_http1_conn;
use base64::engine::general_purpose;
use base64::Engine;
use bytes::Bytes;
use core::str;
use http_body_util::BodyExt;
use hyper::body::Body;
use hyper::client::conn::http1::SendRequest;
use hyper::{Request, StatusCode, Uri};
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

pub struct HttpProxyClient<B> {
    address: String,
    username: String,
    password: String,
    conn: Option<Arc<Mutex<SendRequest<B>>>>,
    last_authority: String,
}

impl<B: Body + 'static + Unpin + Send> HttpProxyClient<B> {
    pub fn new(address: String, username: String, password: String) -> Self {
        HttpProxyClient {
            address,
            username,
            password,
            conn: None,
            last_authority: String::new(),
        }
    }

    pub async fn request(
        &mut self,
        uri: &Uri,
        request: Request<B>,
    ) -> Result<(StatusCode, bytes::Bytes), anyhow::Error>
    where
        B::Data: Send,
        B::Error: Into<Box<dyn Error + Send + Sync>>,
    {
        self.ensure_connection(uri).await?;

        if let Some(c) = &self.conn {
            let mut conn = c.lock().await;
            let resp = conn.send_request(request).await?;
            let status = resp.status();
            let collected = resp.into_body().collect().await?;
            return Ok((status, collected.to_bytes()));
        }

        Err(anyhow::anyhow!("Couldn't find a connection"))
    }

    pub async fn request_many(
        &mut self,
        uri: &Uri,
        requests: Vec<Request<B>>,
    ) -> Result<Vec<(StatusCode, Bytes)>, anyhow::Error>
    where
        B: Body + 'static + Unpin + Send,
        B::Data: Send,
        B::Error: Into<Box<dyn Error + Send + Sync>>,
    {
        self.ensure_connection(uri).await?;

        if let Some(conn) = &self.conn {
            let mut join_set: JoinSet<Result<(StatusCode, Bytes), anyhow::Error>> = JoinSet::new();

            for request in requests {
                let conn_clone = conn.clone();
                join_set.spawn(async move {
                    let mut conn = conn_clone.lock().await;
                    let resp = conn.send_request(request).await?;
                    let status = resp.status();
                    let collected = resp.into_body().collect().await?;
                    Ok((status, collected.to_bytes()))
                });
            }

            let mut results = Vec::with_capacity(join_set.len());
            while let Some(result) = join_set.join_next().await {
                results.push(result??);
            }

            Ok(results)
        } else {
            Err(anyhow::anyhow!("No active connection"))
        }
    }

    async fn ensure_connection(&mut self, uri: &Uri) -> Result<(), anyhow::Error>
    where
        B::Data: Send,
        B::Error: Into<Box<dyn Error + Send + Sync>>,
    {
        if self.conn.is_none() || !self.last_authority.eq(uri.authority().unwrap().as_str()) {
            self.refresh_connection(uri).await?;
        }

        if let Some(c) = &self.conn {
            if c.lock().await.is_closed() {
                self.refresh_connection(uri).await?;
            }
        }
        Ok(())
    }

    async fn refresh_connection(&mut self, uri: &Uri) -> Result<(), anyhow::Error>
    where
        B::Data: Send,
        B::Error: Into<Box<dyn Error + Send + Sync>>,
    {
        let tcp_stream = self.create_tunnel(uri).await?;
        let conn = establish_http1_conn(tcp_stream, uri.host().unwrap().to_string()).await?;
        self.conn = Some(Arc::new(Mutex::new(conn)));
        self.last_authority = uri.authority().unwrap().to_string();
        Ok(())
    }

    async fn create_tunnel(&self, uri: &Uri) -> Result<TcpStream, anyhow::Error> {
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
        if !response_str.contains("200 OK") {
            return Err(anyhow::anyhow!(
                "Failed to establish tunnel: {}",
                response_str
            ));
        }

        Ok(tcp_stream)
    }
}
