#![deny(warnings)]

mod database;
mod http;
mod session;

use database::http as db;
use log::{debug, info};
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error(transparent)]
    Database(#[from] database::traits::Error),
    #[error(transparent)]
    Http(#[from] http::traits::Error),
    #[error(transparent)]
    Session(#[from] session::traits::Error),
    #[error(transparent)]
    Env(#[from] env::VarError),
    #[error(transparent)]
    Parse(#[from] std::num::ParseIntError),
}

const ENV_SESSION_TOKEN: &str = "SESSION_TOKEN";
const ENV_PORT: &str = "PORT";
const DEFAULT_PORT: u16 = 8888;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    debug!("creating database");
    let db = db::Database::new()?;

    let token = env::var(ENV_SESSION_TOKEN)?;
    debug!("create session manager");
    let sessions = session::manager::SessionManager::new(&token)?;

    let port = env::var(ENV_PORT).ok();
    let port = match port {
        Some(port) => port.parse::<u16>()?,
        None => DEFAULT_PORT,
    };
    info!("running server with port {}", port);
    http::server::run(db, sessions, true, port).await?;
    debug!("server stopped");

    Ok(())
}
