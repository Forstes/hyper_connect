use super::send_request::SendRequestEnum;
#[cfg(feature = "http1")]
use crate::connectors::http1::SimpleHttp1Connector;
#[cfg(feature = "http1_proxy")]
use crate::connectors::http1_proxy::ProxyHttp1Connector;
#[cfg(feature = "http2")]
use crate::connectors::http2::SimpleHttp2Connector;
#[cfg(feature = "http2_proxy")]
use crate::connectors::http2_proxy::ProxyHttp2Connector;
use hyper::{body::Body, Uri};

pub enum ConnectorEnum {
    #[cfg(feature = "http1")]
    Http1(SimpleHttp1Connector),
    #[cfg(feature = "http1_proxy")]
    Http1Proxy(ProxyHttp1Connector),
    #[cfg(feature = "http2")]
    Http2(SimpleHttp2Connector),
    #[cfg(feature = "http2_proxy")]
    Http2Proxy(ProxyHttp2Connector),
}

impl ConnectorEnum {
    pub async fn create_connection<B>(&self, uri: &Uri) -> Result<SendRequestEnum<B>, anyhow::Error>
    where
        B: Body + 'static + Unpin + Send,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        match self {
            #[cfg(feature = "http1")]
            ConnectorEnum::Http1(connector) => connector.create_connection(uri).await,
            #[cfg(feature = "http1_proxy")]
            ConnectorEnum::Http1Proxy(connector) => {
                connector
                    .create_connection(uri, &connector.proxy_address, &connector.username, &connector.password)
                    .await
            }
            #[cfg(feature = "http2")]
            ConnectorEnum::Http2(connector) => connector.create_connection(uri).await,
            #[cfg(feature = "http2_proxy")]
            ConnectorEnum::Http2Proxy(connector) => {
                connector
                    .create_connection(uri, &connector.proxy_address, &connector.username, &connector.password)
                    .await
            }
        }
    }
}
