use hyper::Uri;
use hyper_connect::connectors::http1::SimpleHttp1Connector;
use hyper_connect::connectors::types::{Http1Connection, Http1Connector, Http1Sender};
use hyper_connect::handlers::http1_conn_pool::Http1ConnPool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct CountingConnector {
    real_connector: SimpleHttp1Connector,
    count: Arc<AtomicUsize>,
}

impl CountingConnector {
    pub fn new(count: Arc<AtomicUsize>) -> Self {
        Self {
            real_connector: SimpleHttp1Connector {},
            count,
        }
    }
}

impl Http1Connector for CountingConnector {
    async fn create_connection(&self, uri: &Uri) -> Result<(Http1Sender, Http1Connection), anyhow::Error> {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.real_connector.create_connection(uri).await
    }
}

#[tokio::test]
async fn http1_conn_reuse() {
    let count = Arc::new(AtomicUsize::new(0));
    let connector = CountingConnector::new(count.clone());
    let pool = Http1ConnPool::new(connector, 2);

    let uri: Uri = "http://example.com".parse().unwrap();

    // First request -> creates connection
    let conn1 = pool.get_conn(&uri).await.unwrap();
    pool.return_conn(conn1).await;

    // Second request -> should reuse connection
    let conn2 = pool.get_conn(&uri).await.unwrap();
    pool.return_conn(conn2).await;

    // Only one connection created
    assert_eq!(count.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn http1_two_connections_parallel() {
    let count = Arc::new(AtomicUsize::new(0));
    let connector = CountingConnector::new(count.clone());
    let pool = Arc::new(Http1ConnPool::new(connector, 2));

    let uri: Uri = "http://example.com".parse().unwrap();

    let p1 = pool.clone();
    let p2 = pool.clone();

    let uri_clone = uri.clone();
    let t1 = tokio::spawn(async move { p1.get_conn(&uri_clone).await.unwrap() });

    let uri_clone = uri.clone();
    let t2 = tokio::spawn(async move { p2.get_conn(&uri_clone).await.unwrap() });

    let (conn1, conn2) = tokio::join!(t1, t2);

    pool.return_conn(conn1.unwrap()).await;
    pool.return_conn(conn2.unwrap()).await;

    assert_eq!(count.load(Ordering::Relaxed), 2);
}
