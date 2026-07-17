use crate::{clients::request::Request, handlers::traits::HttpHandler};

pub trait HttpClient {
    type Handler: HttpHandler;

    fn get<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Handler>;
    fn post<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Handler>;
    fn put<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Handler>;
    fn delete<'a>(&'a self, uri: &'a str) -> Request<'a, Self::Handler>;
}
