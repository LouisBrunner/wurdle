use hyper::{Body, Request, Response};
use log::info;
use std::convert::Infallible;
use swagger::{Has, XSpanIdString};

pub struct Logger<T> {
    inner: T,
}

pub trait Handler<Request, Response> {
    fn call(self, req: Request) -> Result<Response, Infallible>;
}

impl<T, C> Handler<(Request<Body>, C), Response<Body>> for Logger<T>
where
    T: Handler<(Request<Body>, C), Response<Body>>,
    C: Has<XSpanIdString>,
{
    fn call(self, req: (Request<Body>, C)) -> Result<Response<Body>, Infallible> {
        info!("[REQ] {:?}", req.0);
        self.inner.call(req).map(|res| {
            info!("[RES] {:?}", res);
            res
        })
    }
}
