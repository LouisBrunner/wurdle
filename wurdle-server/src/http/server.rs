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

const UNKNOWN_ERROR: &str = "abe15c99-eaa4-4fb0-a657-b88430fb8910";

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

    fn session_for_word_id(
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
                    id: UNKNOWN_ERROR.to_string(),
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
        match self.sessions.serialize(&session) {
            Ok(session_id) => Ok(wurdle_openapi::models::SessionStart {
                session_id,
                word_id: word_id.to_string(),
            }),
            Err(err) => Err(wurdle_openapi::models::Error {
                id: UNKNOWN_ERROR.to_string(),
                message: format!("{}", err),
                details: None,
            }),
        }
    }

    fn get_session(
        &self,
        session_id: &str,
    ) -> Result<session::session::Session, wurdle_openapi::models::Error> {
        match self.sessions.deserialize(&session_id) {
            Ok(session) => Ok(session),
            // TODO: wrong
            Err(err) => Err(wurdle_openapi::models::Error {
                id: UNKNOWN_ERROR.to_string(),
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

        let mut session = match self.get_session(&inline_object2.session_id) {
            Ok(session) => session,
            Err(err) => {
                // TODO: wrong
                return Ok(wurdle_openapi::DoGuessResponse::InvalidGuess(err));
            }
        };
        Ok(match session.status {
            session::session::Status::InProgress { used_guesses } => {
                let exists = match self.db.guess_exists(&inline_object2.guess) {
                    Ok(exists) => exists,
                    Err(err) => {
                        // TODO: wrong
                        return Ok(wurdle_openapi::DoGuessResponse::ServerError(
                            wurdle_openapi::models::Error {
                                id: UNKNOWN_ERROR.to_string(),
                                message: format!("{}", err),
                                details: None,
                            },
                        ));
                    }
                };

                if !exists {
                    return Ok(wurdle_openapi::DoGuessResponse::InvalidGuess(
                        wurdle_openapi::models::Error {
                            id: UNKNOWN_ERROR.to_string(),
                            message: "word not on the list".to_string(),
                            details: None,
                        },
                    ));
                }

                let word = match self.db.word_for_id(&session.word_id) {
                    Ok(word) => word,
                    Err(err) => {
                        // TODO: WRONG
                        return Ok(wurdle_openapi::DoGuessResponse::InvalidGuess(
                            wurdle_openapi::models::Error {
                                id: UNKNOWN_ERROR.to_string(),
                                message: format!("{}", err),
                                details: None,
                            },
                        ));
                    }
                };

                let session_id = match self.sessions.serialize(&session) {
                    Ok(session_id) => session_id,
                    Err(err) => {
                        return Ok(wurdle_openapi::DoGuessResponse::ServerError(
                            wurdle_openapi::models::Error {
                                id: UNKNOWN_ERROR.to_string(),
                                message: format!("{}", err),
                                details: None,
                            },
                        ))
                    }
                };

                let used_guesses = used_guesses + 1;

                let mut result: Vec<String> = vec![];
                if word.word == inline_object2.guess {
                    session.status = session::session::Status::Won { used_guesses };
                    result = vec![
                        "valid".to_string(),
                        "valid".to_string(),
                        "valid".to_string(),
                        "valid".to_string(),
                        "valid".to_string(),
                    ];
                } else {
                    session.status = if used_guesses >= MAX_GUESSES {
                        session::session::Status::Failed
                    } else {
                        session::session::Status::InProgress { used_guesses }
                    };
                    for (expected, received) in word
                        .word
                        .as_bytes()
                        .iter()
                        .zip(inline_object2.guess.as_bytes().iter())
                    {
                        if expected == received {
                            result.push("valid".to_string())
                        } else if word.word.contains(*received as char) {
                            // TODO: wrong, should count how many times we report that
                            result.push("wrong_place".to_string())
                        } else {
                            result.push("wrong".to_string())
                        }
                    }
                }

                wurdle_openapi::DoGuessResponse::ValidGuess(
                    wurdle_openapi::models::InlineResponse200 {
                        guess_number: used_guesses.into(),
                        status: session.status.to_string(),
                        result,
                        session_id,
                    },
                )
            }
            session::session::Status::Failed | session::session::Status::Won { .. } => {
                wurdle_openapi::DoGuessResponse::InvalidGuess(wurdle_openapi::models::Error {
                    id: UNKNOWN_ERROR.to_string(),
                    message: "session is already finished".to_string(),
                    details: None,
                })
            }
        })
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

        let session = match self.get_session(&session_id) {
            Ok(session) => session,
            Err(err) => {
                // TODO: wrong
                return Ok(wurdle_openapi::GetSessionInfoResponse::UsingAnInvalidSession(err));
            }
        };
        Ok(wurdle_openapi::GetSessionInfoResponse::SuccessfulOperation(
            wurdle_openapi::models::InlineResponse2001 {
                word_id: session.word_id.to_string(),
                status: session.status.to_string(),
                guess_number: match session.status {
                    session::session::Status::InProgress { used_guesses }
                    | session::session::Status::Won { used_guesses } => used_guesses,
                    session::session::Status::Failed => MAX_GUESSES,
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
                        id: UNKNOWN_ERROR.to_string(),
                        message: format!("{}", err),
                        details: None,
                    },
                ))
            }
        };
        let n: usize = rng.gen_range(0..word_length);
        Ok(match self.session_for_word_id(n.to_string().as_str()) {
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

        Ok(
            match self.session_for_word_id(inline_object.word_id.as_str()) {
                Ok(inner) => match inner {
                    Ok(session) => {
                        wurdle_openapi::StartWithIDResponse::SessionCreatedSuccessfully(session)
                    }
                    Err(err) => wurdle_openapi::StartWithIDResponse::ServerError(err),
                },
                Err(err) => wurdle_openapi::StartWithIDResponse::InvalidID(err),
            },
        )
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
                        id: UNKNOWN_ERROR.to_string(),
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
