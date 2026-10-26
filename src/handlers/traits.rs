use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::{header::HeaderMap, Request, StatusCode, Uri};

pub trait HttpHandler {
    fn request(
        &self,
        uri: &Uri,
        request: Request<Either<Full<Bytes>, Empty<Bytes>>>,
    ) -> impl std::future::Future<Output = Result<(StatusCode, HeaderMap, Bytes), anyhow::Error>> + Send;
}
