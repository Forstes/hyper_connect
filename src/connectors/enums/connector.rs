use super::send_request::SendRequestEnum;
use crate::connectors::http2::SimpleHttp2Connector;
use hyper::{body::Body, Uri};

pub enum ConnectorEnum {
    Http2(SimpleHttp2Connector),
}

impl ConnectorEnum {
    pub async fn create_connection<B>(&self, uri: &Uri) -> Result<SendRequestEnum<B>, anyhow::Error>
    where
        B: Body + 'static + Unpin + Send,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        match self {
            ConnectorEnum::Http2(connector) => connector.create_connection(uri).await,
        }
    }
}
