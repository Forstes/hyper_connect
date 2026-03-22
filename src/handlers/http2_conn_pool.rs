use crate::connectors::types::{Http2Connector, Http2Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
struct ConnEntry {
    sender: Arc<tokio::sync::Mutex<Option<Http2Sender>>>,
    is_alive: Arc<AtomicBool>,
}

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

    pub async fn get_conn(&self, uri: &hyper::Uri) -> Result<Http2Sender, anyhow::Error> {
        let authority = uri.authority().ok_or_else(|| anyhow::anyhow!("missing authority"))?.as_str().to_string();

        // Fast map access (short lock)
        let entry = {
            let mut map = self.conns.lock().unwrap();
            map.entry(authority)
                .or_insert_with(|| ConnEntry {
                    sender: Arc::new(tokio::sync::Mutex::new(None)),
                    is_alive: Arc::new(AtomicBool::new(false)),
                })
                .clone()
        };

        {
            let sender = entry.sender.lock().await;
            if let Some(s) = sender.as_ref() {
                if entry.is_alive.load(Ordering::Acquire) {
                    return Ok(s.clone());
                }
            }
        }

        // Create connection without lock
        let (new_sender, connection) = self.connector.create_connection(uri).await?;

        // Only one connection stored
        let mut sender = entry.sender.lock().await;

        if let Some(existing) = sender.as_ref() {
            if entry.is_alive.load(Ordering::Acquire) {
                return Ok(existing.clone()); // discard new
            }
        }

        let entry_clone = entry.clone();
        tokio::task::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("HTTP/2 connection error: {}", e);
            }

            entry_clone.is_alive.store(false, Ordering::Release);
            let mut lock = entry_clone.sender.lock().await;
            *lock = None;
        });

        *sender = Some(new_sender.clone());
        entry.is_alive.store(true, Ordering::Release);

        Ok(new_sender)
    }
}
