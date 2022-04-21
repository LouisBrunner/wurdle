// FIXME: re-enable later
// #![deny(warnings)]

mod database;
mod http;

use database::http as db;
use log::{debug, info};
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error(transparent)]
    Database(#[from] database::traits::Error),
    #[error(transparent)]
    Http(#[from] http::traits::Error),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    debug!("creating database");
    let db = db::Database::new()?;

    let port = 8888;
    info!("running server with port {}", port);
    http::server::run(db, true, port).await?;
    debug!("server stopped");

    Ok(())
}