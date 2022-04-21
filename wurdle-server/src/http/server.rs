use super::traits;
use crate::database::traits::Database;
use hyper::Server;
use log::info;
use std::net::SocketAddr;
use swagger::{ApiError, EmptyContext, Has, XSpanIdString};

use wurdle_openapi;
use wurdle_openapi::models;
use wurdle_openapi::server;

use crate::session;

pub async fn run<T: 'static + Database + Send + Sync + Clone>(
    db: T,
    sessions: session::manager::SessionManager,
    local_server: bool,
    port: u16,
) -> Result<(), traits::Error> {
    let api = Api::new(db, sessions);

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
    sessions: session::manager::SessionManager,
}

impl<T: Database + Send + Sync + Clone> Api<T> {
    fn new(db: T, sessions: session::manager::SessionManager) -> Self {
        Self { db, sessions }
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

        let word_id = inline_object.word_id;
        match self.db.word_for_id(&word_id) {
            Ok(_) => (),
            Err(err) => {
                return Ok(wurdle_openapi::StartWithIDResponse::InvalidID(
                    wurdle_openapi::models::Error {
                        id: "abc".to_string(),
                        message: format!("{}", err),
                        details: None,
                    },
                ))
            }
        };
        let session = session::session::Session::new(&word_id);
        Ok(match self.sessions.serialize(session) {
            Ok(session_id) => wurdle_openapi::StartWithIDResponse::SessionCreatedSuccessfully(
                wurdle_openapi::models::SessionStart {
                    session_id,
                    word_id,
                },
            ),
            Err(err) => {
                wurdle_openapi::StartWithIDResponse::ServerError(wurdle_openapi::models::Error {
                    id: "abc".to_string(),
                    message: format!("{}", err),
                    details: None,
                })
            }
        })
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
