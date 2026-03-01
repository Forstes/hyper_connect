use crate::handlers::http1_conn_pool::Http1ConnPool;
use crate::{connectors::types::Http1Connector, handlers::traits::HttpHandler};
use bytes::Bytes;
use http_body_util::{BodyExt, Either, Empty, Full};
use hyper::{Request, StatusCode, Uri};

pub struct Http1Handler<CN: Http1Connector> {
    conn_pool: Http1ConnPool<CN>,
}

impl<CN: Http1Connector> Http1Handler<CN> {
    pub fn new(conn_pool: Http1ConnPool<CN>) -> Self {
        Self { conn_pool }
    }
}

impl<CN: Http1Connector + Send + Sync> HttpHandler for Http1Handler<CN> {
    async fn request(&self, uri: &Uri, request: Request<Either<Full<Bytes>, Empty<Bytes>>>) -> Result<(StatusCode, Bytes), anyhow::Error> {
        let mut conn = self.conn_pool.get_conn(uri).await?;

        let resp = conn.send_request(request).await?;
        let status = resp.status();
        let collected = resp.into_body().collect().await?;

        self.conn_pool.return_conn(uri, conn).await;

        Ok((status, collected.to_bytes()))
    }
}
