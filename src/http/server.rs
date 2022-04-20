use super::super::database::traits::Database;
use log::debug;

pub struct Server<T: Database> {
    db: T,
    port: u16,
}

impl<T> Server<T>
where
    T: Database,
{
    pub fn new(db: T, port: u16) -> Self {
        Self { db, port }
    }

    pub fn run(&self) {
        loop {
            match self.db.word_length() {
                Ok(len) => debug!("words length: {}", len),
                Err(_err) => (),
            };
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
