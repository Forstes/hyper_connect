use crate::connectors::types::{Http1Connection, Http1Connector};
use hyper::Uri;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub struct Http1ConnPool<CN: Http1Connector> {
    connector: CN,
    max_conns_per_host: usize,
    pool: Arc<Mutex<HashMap<String, Vec<Http1Connection>>>>,
}

impl<CN: Http1Connector> Http1ConnPool<CN> {
    pub fn new(connector: CN, max_conns_per_host: usize) -> Self {
        Self {
            connector,
            max_conns_per_host,
            pool: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_conn(&self, uri: &Uri) -> Result<Http1Connection, anyhow::Error> {
        let authority = uri.authority().ok_or_else(|| anyhow::anyhow!("Missing authority"))?.as_str().to_string();

        // Try take existing connection
        {
            let mut pool = self.pool.lock().await;

            if let Some(conns) = pool.get_mut(&authority) {
                while let Some(conn) = conns.pop() {
                    if !conn.is_closed() {
                        return Ok(conn);
                    }
                    // Drop closed connection silently
                }
            }
        }

        // No available connection -> create new
        self.connector.create_connection(uri).await
    }

    /// Return connection back to pool
    pub async fn return_conn(&self, uri: &Uri, conn: Http1Connection) {
        if conn.is_closed() {
            return;
        }

        let authority = match uri.authority() {
            Some(a) => a.as_str().to_string(),
            None => return,
        };

        let mut pool = self.pool.lock().await;
        let conns = pool.entry(authority).or_default();

        if conns.len() < self.max_conns_per_host {
            conns.push(conn);
        }
        // else: drop it
    }
}
