use crate::connectors::types::Http2Connector;
use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::client::conn::http2::SendRequest;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

type ConnEntry = Arc<tokio::sync::Mutex<Option<SendRequest<Either<Full<Bytes>, Empty<Bytes>>>>>>;

pub struct Http2ConnPool<CN>
where
    CN: Http2Connector,
{
    connector: CN,
    conns: Mutex<HashMap<String, ConnEntry>>,
}

impl<CN> Http2ConnPool<CN>
where
    CN: Http2Connector,
{
    pub fn new(connector: CN) -> Self {
        Self {
            connector,
            conns: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get_conn(&self, uri: &hyper::Uri) -> Result<SendRequest<Either<Full<Bytes>, Empty<Bytes>>>, anyhow::Error> {
        let authority = uri.authority().ok_or_else(|| anyhow::anyhow!("missing authority"))?.as_str().to_string();

        // Fast map access (short lock)
        let entry = {
            let mut map = self.conns.lock().unwrap();
            map.entry(authority.clone())
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(None)))
                .clone()
        };

        // Per-authority lock
        let mut entry = entry.lock().await;

        if let Some(conn) = entry.as_ref() {
            if !conn.is_closed() {
                return Ok(conn.clone());
            }
        }

        // Create connection (only one task per authority reaches here)
        let new_conn = self.connector.create_connection(uri).await?;

        *entry = Some(new_conn.clone());

        Ok(new_conn)
    }
}
