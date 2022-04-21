use hyper::service::Service;
use log::info;
use std::task::{Context, Poll};

pub struct Logger<T> {
    inner: T,
}

impl<T> Logger<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T, Request> Service<Request> for Logger<T>
where
    T: Service<Request>,
    Request: core::fmt::Debug,
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = T::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        info!("[REQ] {:?}", request);
        self.inner.call(request)
    }
}
