use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::{Request, StatusCode, Uri};

pub trait HttpHandler {
    async fn request(&self, uri: &Uri, request: Request<Either<Full<Bytes>, Empty<Bytes>>>) -> Result<(StatusCode, Bytes), anyhow::Error>;
}
