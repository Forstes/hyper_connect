use super::conn_pool::ConnectionPool;
use crate::connection::HttpConnection;
use crate::connectors::HttpConnector;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::body::Body;
use hyper::{Request, StatusCode, Uri};
use tokio::task::JoinSet;

/// Generic handler for connecton & request sending
pub struct HttpHandler<B, CN>
where
    B: Body + 'static + Unpin + Send + Sync,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    CN: HttpConnector,
{
    connector: CN,
    connection_pool: ConnectionPool<B, CN>,
}

impl<B, CN> HttpHandler<B, CN>
where
    B: Body + 'static + Unpin + Send + Sync,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    CN: HttpConnector,
{
    pub fn new(connector: CN, connection_pool: ConnectionPool<B, CN>) -> Self {
        Self {
            connector,
            connection_pool,
        }
    }

    pub async fn request(&self, uri: &Uri, request: Request<B>) -> Result<(StatusCode, Bytes), anyhow::Error> {
        let conn = self.connection_pool.get_conn(uri, &self.connector).await?;

        let mut conn = conn.write().await;
        let resp = conn.send_request(request).await?;
        let status = resp.status();
        let collected = resp.into_body().collect().await?;
        return Ok((status, collected.to_bytes()));
    }

    pub async fn request_many(&self, uri: &Uri, requests: Vec<Request<B>>) -> Result<Vec<(StatusCode, Bytes)>, anyhow::Error> {
        let conn = self.connection_pool.get_conn(uri, &self.connector).await?;

        let mut join_set: JoinSet<Result<(StatusCode, Bytes), anyhow::Error>> = JoinSet::new();

        for request in requests {
            let conn_clone = conn.clone();
            join_set.spawn(async move {
                let mut conn = conn_clone.write().await;
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
    }
}
