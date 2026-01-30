use bytes::Bytes;
use http_body_util::Empty;
use hyper::{body::Body, Request, Response, Uri};
use hyper_connect::{connection::HttpConnection, connectors::HttpConnector, handlers::conn_pool::ConnectionPool};
use mockall::mock;
use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

mock! {
    pub HttpConn<B: Body + 'static> {}

    impl<B: Body + Send + 'static> HttpConnection<B> for HttpConn<B> {
        fn is_conn_ready(&self) -> bool;
        fn is_conn_closed(&self) -> bool;
        async fn send_request(&mut self, req: Request<B>) -> Result<Response<hyper::body::Incoming>, hyper::Error>;
    }
}

mock! {
    pub Connector {}

    impl HttpConnector for Connector {
        type Connection<B> = MockHttpConn<B>
        where
        B: Body + Unpin + Send + Sync + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>;

        async fn create_connection<B>(
            &self,
            uri: &Uri,
        ) -> Result<<MockConnector as HttpConnector>::Connection<B>, anyhow::Error>
        where
            B: hyper::body::Body + 'static + Unpin + Send + Sync,
            B::Data: Send,
            B::Error: Into<Box<dyn std::error::Error + Send + Sync>>;

    }
}

#[tokio::test]
async fn test_connection_pool_uses_existing_ready_connection() {
    let uri = Uri::from_str("http://example.com").unwrap();

    // Counter to verify how many connections were created
    let conn_counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = conn_counter.clone();

    let mut mock_connector = MockConnector::new();

    // Expect exactly 1 connection to be created
    mock_connector
        .expect_create_connection::<Empty<Bytes>>()
        .returning(move |_uri| {
            let mut mock_conn = MockHttpConn::new();

            mock_conn.expect_is_conn_ready().return_const(true);
            mock_conn.expect_is_conn_closed().return_const(false);

            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(mock_conn)
        });

    let pool = ConnectionPool::<Empty<Bytes>, MockConnector>::new(mock_connector, 2);

    // First connection (will trigger connector)
    let conn1 = pool.get_conn(&uri).await.unwrap();

    // Second request (should reuse the first one)
    let conn2 = pool.get_conn(&uri).await.unwrap();

    // Should not have created a second connection
    assert_eq!(conn_counter.load(Ordering::SeqCst), 1);

    // Should be the same Arc (reused)
    assert!(Arc::ptr_eq(&conn1, &conn2));
}

#[tokio::test]
async fn test_connection_pool_creates_new_connection() {
    let uri = Uri::from_str("http://example.com").unwrap();

    // Counter to verify how many connections were created
    let conn_counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = conn_counter.clone();

    let mut mock_connector = MockConnector::new();

    // Expect exactly 2 connections to be created
    mock_connector
        .expect_create_connection::<Empty<Bytes>>()
        .returning(move |_uri| {
            let mut mock_conn = MockHttpConn::new();

            mock_conn.expect_is_conn_ready().return_const(true);
            mock_conn.expect_is_conn_closed().return_const(false);

            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(mock_conn)
        });

    let pool = ConnectionPool::<Empty<Bytes>, MockConnector>::new(mock_connector, 2);

    // First connection (will trigger connector)
    let conn1 = pool.get_conn(&uri).await.unwrap();

    // Simulate first connection is in use
    let locked_conn = conn1.write().await;

    // Second request (should create new connection)
    let conn2 = pool.get_conn(&uri).await.unwrap();

    drop(locked_conn);

    // Should not have created a second connection
    assert_eq!(conn_counter.load(Ordering::SeqCst), 2);

    assert!(!Arc::ptr_eq(&conn1, &conn2));
}

#[tokio::test]
async fn test_connection_pool_waits_until_connection_released() {
    use std::time::Duration;
    use tokio::time::timeout;

    let uri = Uri::from_str("http://example.com").unwrap();

    let conn_counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = conn_counter.clone();

    let mut mock_connector = MockConnector::new();

    // Mock to return one connection and count creations
    mock_connector
        .expect_create_connection::<Empty<Bytes>>()
        .returning(move |_uri| {
            let mut mock_conn = MockHttpConn::new();
            mock_conn.expect_is_conn_ready().return_const(true);
            mock_conn.expect_is_conn_closed().return_const(false);
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(mock_conn)
        });

    let pool = ConnectionPool::<Empty<Bytes>, MockConnector>::new(mock_connector, 1);

    // First connection: acquired and locked
    let conn1 = pool.get_conn(&uri).await.unwrap();
    let conn1_lock = conn1.write().await;

    // Second connection: start waiting (it will block)
    let get_conn2 = pool.get_conn(&uri);
    tokio::pin!(get_conn2);

    // Wait a short time to ensure it's waiting, not resolving
    let result = timeout(Duration::from_millis(50), &mut get_conn2).await;
    assert!(result.is_err(), "Expected second connection to be blocked");

    // Still only one connection created
    assert_eq!(conn_counter.load(Ordering::SeqCst), 1);

    // Now release the first connection
    drop(conn1_lock);

    // Second connection should now succeed
    let conn2 = get_conn2.await.unwrap();

    // Still only one connection created
    assert_eq!(conn_counter.load(Ordering::SeqCst), 1);

    // And it's the same underlying connection
    assert!(Arc::ptr_eq(&conn1, &conn2));
}

#[tokio::test]
async fn test_closed_connection_is_removed_from_pool() {
    let uri = Uri::from_static("http://example.com");

    let conn_counter = Arc::new(AtomicUsize::new(0));
    let counter_clone1 = conn_counter.clone();
    let counter_clone2 = conn_counter.clone();

    let mut mock_connector = MockConnector::new();

    // First call: return a closed connection
    let mut call_count = 0;
    mock_connector
        .expect_create_connection::<Empty<Bytes>>()
        .returning(move |_uri| {
            let counter = if call_count == 0 { &counter_clone1 } else { &counter_clone2 };
            call_count += 1;

            let mut mock_conn = MockHttpConn::new();
            mock_conn.expect_is_conn_ready().return_const(true);
            mock_conn.expect_is_conn_closed().return_const(call_count == 1); // First connection is closed

            counter.fetch_add(1, Ordering::SeqCst);
            Ok(mock_conn)
        });

    // Pool with limit 1
    let pool = ConnectionPool::<Empty<Bytes>, MockConnector>::new(mock_connector, 1);

    // First call inserts a closed connection (should trigger cleanup)
    let conn1 = pool.get_conn(&uri).await.unwrap();

    assert_eq!(conn1.read().await.is_conn_closed(), true);

    // Request again and ensure no new connection is created
    let conn2 = pool.get_conn(&uri).await.unwrap();
    assert_eq!(conn_counter.load(Ordering::SeqCst), 2);

    // Ensure second connection is reused
    assert!(!Arc::ptr_eq(&conn1, &conn2));
}
