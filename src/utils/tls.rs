use std::sync::Arc;
use tokio_rustls::TlsConnector;

pub fn create_tls_connector(is_alpn: bool) -> TlsConnector {
    let root_store = tokio_rustls::rustls::RootCertStore::from_iter(
        webpki_roots::TLS_SERVER_ROOTS.iter().cloned(),
    );

    let mut config = tokio_rustls::rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    if is_alpn {
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    }

    TlsConnector::from(Arc::new(config))
}
