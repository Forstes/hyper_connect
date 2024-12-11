use crate::connectors::enums::connector::ConnectorEnum;
use crate::connectors::enums::send_request::SendRequestEnum;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::body::Body;
use hyper::{Request, StatusCode, Uri};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

/// Generic handler for connecton & request sending
pub struct HttpHandler<B: Body + 'static> {
    connector: ConnectorEnum,
    connection: Option<Arc<Mutex<SendRequestEnum<B>>>>,
    last_authority: String,
}

impl<B> HttpHandler<B>
where
    B: Body + 'static + Unpin + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    pub fn new(connector: ConnectorEnum) -> Self {
        Self {
            connector,
            connection: None,
            last_authority: String::new(),
        }
    }

    pub async fn request(
        &mut self,
        uri: &Uri,
        request: Request<B>,
    ) -> Result<(StatusCode, Bytes), anyhow::Error> {
        self.ensure_connection(uri).await?;

        if let Some(c) = &self.connection {
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

        if let Some(conn) = &self.connection {
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
        if let Some(c) = &self.connection {
            if !self.last_authority.eq(uri.authority().unwrap().as_str())
                || !c.lock().await.is_conn_ready()
            {
                self.connection = Some(Arc::new(Mutex::new(
                    self.connector.create_connection(uri).await?,
                )));
                self.last_authority = uri.authority().unwrap().as_str().to_string();
            }
        } else {
            self.connection = Some(Arc::new(Mutex::new(
                self.connector.create_connection(uri).await?,
            )));
        }
        Ok(())
    }
}
