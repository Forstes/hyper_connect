use hyper::{body::Body, Uri};
use std::{collections::HashMap, future::Future, sync::Arc, time::Duration};
use tokio::sync::RwLock;

pub struct ConnectionPool<B: Body + 'static, C: HttpConnection<B>, CN: HttpConnector> {
    max_conns_per_host: usize,
    pool: Arc<RwLock<HashMap<String, Vec<Arc<RwLock<C>>>>>>,
    _pht1: std::marker::PhantomData<B>,
    _pht2: std::marker::PhantomData<CN>,
}

pub trait HttpConnection<B: Body + 'static> {
    fn send_request(
        &mut self,
        req: hyper::Request<B>,
    ) -> impl Future<Output = Result<hyper::Response<hyper::body::Incoming>, hyper::Error>>;
    fn is_conn_ready(&self) -> bool;
    fn is_conn_closed(&self) -> bool;
}

pub trait HttpConnector {
    fn create_connection<B, T: HttpConnection<B> + 'static>(&self, uri: &Uri) -> impl Future<Output = Result<T, anyhow::Error>>
    where
        B: Body + 'static + Unpin + Send,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>;
}

impl<B, C, CN> ConnectionPool<B, C, CN>
where
    B: Body + 'static + Unpin + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    C: HttpConnection<B> + 'static,
    CN: HttpConnector,
{
    pub fn new(max_conns_per_host: usize) -> Self {
        Self {
            max_conns_per_host,
            pool: Arc::new(RwLock::new(HashMap::new())),
            _pht1: std::marker::PhantomData,
            _pht2: std::marker::PhantomData,
        }
    }

    pub async fn get_conn(&self, uri: &Uri, connector: &CN) -> Result<Arc<RwLock<C>>, anyhow::Error> {
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
