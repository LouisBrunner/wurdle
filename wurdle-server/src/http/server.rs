use super::traits;
use crate::database::traits::Database;
use hyper::Server;
use log::info;
use std::net::SocketAddr;
use swagger::{ApiError, EmptyContext, Has, XSpanIdString};

use wurdle_openapi;
use wurdle_openapi::models;
use wurdle_openapi::server;

pub async fn run<T: 'static + Database + Send + Sync + Clone>(
    db: T,
    local_server: bool,
    port: u16,
) -> Result<(), traits::Error> {
    let api = Api::new(db);

    let addr = match local_server {
        true => [127, 0, 0, 1],
        false => [0, 0, 0, 0],
    };
    let addr = SocketAddr::from((addr, port));

    let service = server::MakeService::new(api);
    let service = server::context::MakeAddContext::<_, EmptyContext>::new(service);

    let server = Server::bind(&addr).serve(service);
    Ok(server.await?)
}

#[derive(Clone)]
struct Api<T: Database + Send + Sync + Clone> {
    db: T,
}

impl<T: Database + Send + Sync + Clone> Api<T> {
    fn new(db: T) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl<C, T> wurdle_openapi::Api<C> for Api<T>
where
    C: Has<XSpanIdString> + Send + Sync,
    T: Database + Send + Sync + Clone,
{
    async fn do_guess(
        &self,
        inline_object2: models::InlineObject2,
        context: &C,
    ) -> Result<wurdle_openapi::DoGuessResponse, ApiError> {
        let context = context.clone();
        info!(
            "do_guess({:?}) - X-Span-ID: {:?}",
            inline_object2,
            context.get().0.clone()
        );
        Err(ApiError("not implemented".to_string()))
    }

    async fn get_session_info(
        &self,
        session_id: String,
        context: &C,
    ) -> Result<wurdle_openapi::GetSessionInfoResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_session_info(\"{}\") - X-Span-ID: {:?}",
            session_id,
            context.get().0.clone()
        );
        Err(ApiError("not implemented".to_string()))
    }

    async fn start_random(
        &self,
        context: &C,
    ) -> Result<wurdle_openapi::StartRandomResponse, ApiError> {
        let context = context.clone();
        info!("start_random() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("not implemented".to_string()))
    }

    async fn start_with_id(
        &self,
        inline_object: models::InlineObject,
        context: &C,
    ) -> Result<wurdle_openapi::StartWithIDResponse, ApiError> {
        let context = context.clone();
        info!(
            "start_with_id({:?}) - X-Span-ID: {:?}",
            inline_object,
            context.get().0.clone()
        );
        Err(ApiError("not implemented".to_string()))
    }

    async fn start_with_word(
        &self,
        inline_object1: models::InlineObject1,
        context: &C,
    ) -> Result<wurdle_openapi::StartWithWordResponse, ApiError> {
        let context = context.clone();
        info!(
            "start_with_word({:?}) - X-Span-ID: {:?}",
            inline_object1,
            context.get().0.clone()
        );
        Err(ApiError("not implemented".to_string()))
    }
}
