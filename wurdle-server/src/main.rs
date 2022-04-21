// FIXME: re-enable later
// #![deny(warnings)]

mod database;
mod http;
mod session;

use database::http as db;
use log::{debug, info};
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error(transparent)]
    Database(#[from] database::traits::Error),
    #[error(transparent)]
    Http(#[from] http::traits::Error),
    #[error(transparent)]
    Session(#[from] session::traits::Error),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    debug!("creating database");
    let db = db::Database::new()?;

    debug!("create session manager");
    let sessions = session::manager::SessionManager::new("TODO")?;

    let port = 8888;
    info!("running server with port {}", port);
    http::server::run(db, sessions, true, port).await?;
    debug!("server stopped");

    Ok(())
}
