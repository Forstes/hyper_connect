use crate::{connection::HttpConnection, connectors::HttpConnector};
use hyper::{body::Body, Uri};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;

pub struct ConnectionPool<B, CN>
where
    B: Body + 'static + Unpin + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    CN: HttpConnector,
    CN::Connection<B>: HttpConnection<B>,
{
    max_conns_per_host: usize,
    pool: Arc<RwLock<HashMap<String, Vec<Arc<RwLock<CN::Connection<B>>>>>>>,
    _pht1: std::marker::PhantomData<B>,
    _pht2: std::marker::PhantomData<CN>,
}

impl<B, CN> ConnectionPool<B, CN>
where
    B: Body + 'static + Unpin + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    CN: HttpConnector,
    CN::Connection<B>: HttpConnection<B>,
{
    pub fn new(max_conns_per_host: usize) -> Self {
        Self {
            max_conns_per_host,
            pool: Arc::new(RwLock::new(HashMap::new())),
            _pht1: std::marker::PhantomData,
            _pht2: std::marker::PhantomData,
        }
    }

    pub async fn get_conn(&self, uri: &Uri, connector: &CN) -> Result<Arc<RwLock<CN::Connection<B>>>, anyhow::Error> {
        let authority = uri.authority().unwrap().to_string();

        loop {
            let mut cleanup_needed = false;
            let mut can_create_new_conn = false;

            {
                let pool = self.pool.read().await;

                if let Some(conns) = pool.get(&authority) {
                    for conn in conns.iter() {
                        if let Ok(guard) = conn.try_read() {
                            if guard.is_conn_closed() {
                                cleanup_needed = true;
                            } else if guard.is_conn_ready() {
                                return Ok(conn.clone());
                            }
                        }
                    }

                    if conns.len() < self.max_conns_per_host {
                        can_create_new_conn = true;
                    }
                } else {
                    // Host entry doesn't exist yet
                    can_create_new_conn = true;
                }
            }

            if cleanup_needed {
                let mut pool = self.pool.write().await;
                if let Some(conns) = pool.get_mut(&authority) {
                    conns.retain(|conn| {
                        if let Ok(guard) = conn.try_read() {
                            !guard.is_conn_closed()
                        } else {
                            true // keep if cannot check
                        }
                    });
                }
            }

            if can_create_new_conn {
                let new_conn = Arc::new(RwLock::new(connector.create_connection(uri).await?));
                let mut pool = self.pool.write().await;
                let conns = pool.entry(authority.clone()).or_insert_with(Vec::new);
                if conns.len() < self.max_conns_per_host {
                    conns.push(new_conn.clone());
                    return Ok(new_conn);
                }
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}
