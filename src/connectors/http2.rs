use crate::tls::create_tls_connector;
use hyper::{
    body::Body,
    client::conn::http1::{self, SendRequest},
    Uri,
};
use hyper_util::rt::TokioIo;
use std::{error::Error, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    time::timeout,
};

pub trait Http2Connector<B>
where
    B: Body + 'static + Unpin + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
{
    async fn create_connection(&self, uri: &Uri) -> Result<SendRequest<B>, anyhow::Error>;
}

pub struct SimpleHttp2Connector {}

impl<B> Http2Connector<B> for SimpleHttp2Connector
where
    B: Body + 'static + Unpin + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    async fn create_connection(&self, uri: &Uri) -> Result<SendRequest<B>, anyhow::Error> {
        todo!()
    }
}
