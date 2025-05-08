use hyper::{
    body::Body,
    client::conn::{http1, http2},
    Request, Response,
};

pub enum SendRequestEnum<B>
where
    B: Body + 'static,
{
    Http1(http1::SendRequest<B>),
    Http2(http2::SendRequest<B>),
}

impl<B> SendRequestEnum<B>
where
    B: Body + 'static,
{
    pub async fn send_request(&mut self, req: Request<B>) -> Result<Response<hyper::body::Incoming>, hyper::Error> {
        match self {
            SendRequestEnum::Http1(ref mut sender) => sender.send_request(req).await,
            SendRequestEnum::Http2(ref mut sender) => sender.send_request(req).await,
        }
    }

    pub fn is_conn_ready(&self) -> bool {
        match self {
            SendRequestEnum::Http1(ref sender) => sender.is_ready(),
            SendRequestEnum::Http2(ref sender) => sender.is_ready(),
        }
    }

    pub fn is_conn_closed(&self) -> bool {
        match self {
            SendRequestEnum::Http1(ref sender) => sender.is_closed(),
            SendRequestEnum::Http2(ref sender) => sender.is_closed(),
        }
    }
}
