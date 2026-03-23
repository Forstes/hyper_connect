use crate::connectors::types::{Http1Connector, Http1Sender};
use hyper::Uri;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::sync::Mutex as AsyncMutex;

struct Conn {
    sender: Http1Sender,
    is_alive: Arc<AtomicBool>,
}

struct ConnEntry {
    conns: AsyncMutex<Vec<Conn>>,
}

pub struct PooledHttp1 {
    pub sender: Http1Sender,
    is_alive: Arc<AtomicBool>,
    entry: Arc<ConnEntry>,
}

pub struct Http1ConnPool<CN: Http1Connector> {
    connector: CN,
    max_conns_per_host: usize,
    pool: Mutex<HashMap<String, Arc<ConnEntry>>>,
}

impl<CN: Http1Connector> Http1ConnPool<CN> {
    pub fn new(connector: CN, max_conns_per_host: usize) -> Self {
        Self {
            connector,
            max_conns_per_host,
            pool: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get_conn(&self, uri: &Uri) -> Result<PooledHttp1, anyhow::Error> {
        let authority = uri.authority().ok_or_else(|| anyhow::anyhow!("Missing authority"))?.as_str().to_string();

        let entry = {
            let mut map = self.pool.lock().unwrap();
            map.entry(authority)
                .or_insert_with(|| {
                    Arc::new(ConnEntry {
                        conns: AsyncMutex::new(Vec::new()),
                    })
                })
                .clone()
        };

        // reuse
        {
            let mut conns = entry.conns.lock().await;

            while let Some(conn) = conns.pop() {
                if conn.is_alive.load(Ordering::Acquire) {
                    return Ok(PooledHttp1 {
                        sender: conn.sender,
                        is_alive: conn.is_alive,
                        entry: entry.clone(),
                    });
                }
            }
        }

        // create
        let (sender, connection) = self.connector.create_connection(uri).await?;

        let is_alive = Arc::new(AtomicBool::new(true));

        let entry_clone = entry.clone();
        let is_alive_clone = is_alive.clone();

        tokio::spawn(async move {
            let res = connection.await;

            if let Err(e) = &res {
                eprintln!("HTTP/1 connection error: {}", e);
            }

            is_alive_clone.store(false, Ordering::Release);

            let mut conns = entry_clone.conns.lock().await;
            conns.retain(|c| !Arc::ptr_eq(&c.is_alive, &is_alive_clone));
        });

        Ok(PooledHttp1 { sender, is_alive, entry })
    }

    pub async fn return_conn(&self, conn: PooledHttp1) {
        if !conn.is_alive.load(Ordering::Acquire) {
            return;
        }

        let mut conns = conn.entry.conns.lock().await;

        if conns.len() < self.max_conns_per_host {
            conns.push(Conn {
                sender: conn.sender,
                is_alive: conn.is_alive,
            });
        }
        // else drop
    }
}
