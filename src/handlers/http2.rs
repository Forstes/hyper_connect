use crate::handlers::http2_conn_pool::Http2ConnPool;
use crate::{connectors::types::Http2Connector, handlers::traits::HttpHandler};
use bytes::Bytes;
use http_body_util::{BodyExt, Either, Empty, Full};
use hyper::{Request, StatusCode, Uri};

pub struct Http2Handler<CN: Http2Connector> {
    conn_pool: Http2ConnPool<CN>,
}

impl<CN: Http2Connector> Http2Handler<CN> {
    pub fn new(conn_pool: Http2ConnPool<CN>) -> Self {
        Self { conn_pool }
    }
}

impl<CN: Http2Connector + Send + Sync> HttpHandler for Http2Handler<CN> {
    async fn request(&self, uri: &Uri, request: Request<Either<Full<Bytes>, Empty<Bytes>>>) -> Result<(StatusCode, Bytes), anyhow::Error> {
        let mut conn = self.conn_pool.get_conn(uri).await?;

        let resp = conn.send_request(request).await?;
        let status = resp.status();
        let collected = resp.into_body().collect().await?;

        Ok((status, collected.to_bytes()))
    }
}
