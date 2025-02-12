use super::send_request::SendRequestEnum;
use crate::connectors::{
    http1::SimpleHttp1Connector, http1_proxy::ProxyHttp1Connector, http2::SimpleHttp2Connector,
    http2_proxy::ProxyHttp2Connector,
};
use hyper::{body::Body, Uri};

pub enum ConnectorEnum {
    Http1(SimpleHttp1Connector),
    Http1Proxy(ProxyHttp1Connector),
    Http2(SimpleHttp2Connector),
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
            ConnectorEnum::Http1(connector) => connector.create_connection(uri).await,
            ConnectorEnum::Http1Proxy(connector) => {
                connector
                    .create_connection(uri, &connector.proxy_address, &connector.username, &connector.password)
                    .await
            }
            ConnectorEnum::Http2(connector) => connector.create_connection(uri).await,
            ConnectorEnum::Http2Proxy(connector) => {
                connector
                    .create_connection(uri, &connector.proxy_address, &connector.username, &connector.password)
                    .await
            }
        }
    }
}
