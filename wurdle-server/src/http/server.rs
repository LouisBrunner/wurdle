use super::traits;
use crate::database::traits::Database;
use hyper::Server;
use log::info;
use rand::{thread_rng, Rng};
use std::net::SocketAddr;
use swagger::{ApiError, EmptyContext, Has, XSpanIdString};

use wurdle_openapi;
use wurdle_openapi::models;
use wurdle_openapi::server;

use crate::session;

const MAX_GUESSES: u8 = 6;

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

    fn word_for_id(
        &self,
        word_id: &str,
    ) -> Result<
        Result<wurdle_openapi::models::SessionStart, wurdle_openapi::models::Error>,
        wurdle_openapi::models::Error,
    > {
        match self.db.word_for_id(&word_id) {
            Ok(_) => (),
            Err(err) => {
                // TODO: WRONG
                return Err(wurdle_openapi::models::Error {
                    id: "abc".to_string(),
                    message: format!("{}", err),
                    details: None,
                });
            }
        };
        Ok(self.make_session(word_id))
    }

    fn make_session(
        &self,
        word_id: &str,
    ) -> Result<wurdle_openapi::models::SessionStart, wurdle_openapi::models::Error> {
        let session = session::session::Session::new(word_id);
        match self.sessions.serialize(session) {
            Ok(session_id) => Ok(wurdle_openapi::models::SessionStart {
                session_id,
                word_id: word_id.to_string(),
            }),
            Err(err) => Err(wurdle_openapi::models::Error {
                id: "abc".to_string(),
                message: format!("{}", err),
                details: None,
            }),
        }
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

        let session = match self.sessions.deserialize(&session_id) {
            Ok(session) => session,
            // TODO: wrong
            Err(err) => {
                return Ok(
                    wurdle_openapi::GetSessionInfoResponse::UsingAnInvalidSession(
                        wurdle_openapi::models::Error {
                            id: "abc".to_string(),
                            message: format!("{}", err),
                            details: None,
                        },
                    ),
                )
            }
        };
        Ok(wurdle_openapi::GetSessionInfoResponse::SuccessfulOperation(
            wurdle_openapi::models::InlineResponse2001 {
                word_id: session.word_id.to_string(),
                status: match session.status {
                    session::session::Status::InProgress { .. } => "in_progress",
                    session::session::Status::Failed => "failed",
                    session::session::Status::Won { .. } => "guessed",
                }
                .to_string(),
                guess_number: match session.status {
                    session::session::Status::InProgress { used_guesses } => used_guesses,
                    session::session::Status::Failed => MAX_GUESSES,
                    session::session::Status::Won { used_guesses } => used_guesses,
                }
                .into(),
            },
        ))
    }

    async fn start_random(
        &self,
        context: &C,
    ) -> Result<wurdle_openapi::StartRandomResponse, ApiError> {
        let context = context.clone();
        info!("start_random() - X-Span-ID: {:?}", context.get().0.clone());

        let mut rng = thread_rng();

        let word_length = match self.db.word_length() {
            Ok(word_length) => word_length,
            Err(err) => {
                return Ok(wurdle_openapi::StartRandomResponse::ServerError(
                    wurdle_openapi::models::Error {
                        id: "abc".to_string(),
                        message: format!("{}", err),
                        details: None,
                    },
                ))
            }
        };
        let n: usize = rng.gen_range(0..word_length);
        Ok(match self.word_for_id(n.to_string().as_str()) {
            Ok(inner) => match inner {
                Ok(session) => {
                    wurdle_openapi::StartRandomResponse::SessionCreatedSuccessfully(session)
                }
                Err(err) => wurdle_openapi::StartRandomResponse::ServerError(err),
            },
            Err(err) => wurdle_openapi::StartRandomResponse::ServerError(err),
        })
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

        Ok(match self.word_for_id(inline_object.word_id.as_str()) {
            Ok(inner) => match inner {
                Ok(session) => {
                    wurdle_openapi::StartWithIDResponse::SessionCreatedSuccessfully(session)
                }
                Err(err) => wurdle_openapi::StartWithIDResponse::ServerError(err),
            },
            Err(err) => wurdle_openapi::StartWithIDResponse::InvalidID(err),
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

        let word = match self.db.word_exists(inline_object1.word.as_str()) {
            Ok(word) => word,
            Err(err) => {
                return Ok(wurdle_openapi::StartWithWordResponse::InvalidWord(
                    wurdle_openapi::models::Error {
                        id: "abc".to_string(),
                        message: format!("{}", err),
                        details: None,
                    },
                ))
            }
        };
        Ok(match self.make_session(word.word_id.as_str()) {
            Ok(session) => {
                wurdle_openapi::StartWithWordResponse::SessionCreatedSuccessfully(session)
            }
            Err(err) => wurdle_openapi::StartWithWordResponse::ServerError(err),
        })
    }
}
