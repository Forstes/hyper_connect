use hyper::{Request, StatusCode, Body, Uri};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use bytes::Bytes;

pub struct HttpClient<B, C>
where
    C: HttpClient<B> + Send + Sync,
{
    client: C,
    conn: Option<Arc<Mutex<hyper::client::conn::http1::SendRequest<B>>>>,
    last_authority: String,
}

impl<B, C> HttpClient<B, C>
where
    B: Body + 'static + Unpin + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    C: HttpClient<B> + Send + Sync,
{
    pub fn new(client: C) -> Self {
        Self {
            client,
            conn: None,
            last_authority: String::new(),
        }
    }

    pub async fn request(
        &mut self,
        uri: &Uri,
        request: Request<B>,
    ) -> Result<(StatusCode, Bytes), anyhow::Error> {
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
    ) -> Result<Vec<(StatusCode, Bytes)>, anyhow::Error> {
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

    async fn ensure_connection(&mut self, uri: &Uri) -> Result<(), anyhow::Error> {
        if self.conn.is_none() || !self.last_authority.eq(uri.authority().unwrap().as_str()) {
            self.client.refresh_connection(uri).await?;
        }

        if let Some(c) = &self.conn {
            if c.lock().await.is_closed() {
                self.client.refresh_connection(uri).await?;
            }
        }
        Ok(())
    }
}
