mod database;
mod http;

use database::http as db;
use log::{debug, info};
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error(transparent)]
    Database(#[from] database::traits::Error),
}

fn main() -> Result<(), Error> {
    env_logger::init();

    debug!("creating database");
    let db = db::Database::new()?;

    let port = 8888;
    info!("creating server with port {}", port);
    let server = http::server::Server::new(db, port);

    debug!("running server");
    server.run();
    debug!("server stopped");

    Ok(())
}
