use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("out of bounds: {index} is outside {maximum}")]
    OutOfBounds { index: usize, maximum: usize },
    #[error("missing word: {word}")]
    MissingWord { word: String },
    #[error("invalid id: {id}")]
    InvalidID { id: String },
}

pub struct Word {
    pub word_id: String,
    pub word: String,
}

pub trait Database {
    // For specific words
    fn word_for_id(&self, id: &str) -> Result<Word, Error>;
    fn word_exists(&self, word: &str) -> Result<Word, Error>;
    // For random
    fn word_length(&self) -> Result<usize, Error>;
    fn word_for_index(&self, id: usize) -> Result<Word, Error>;
    // For guessing
    fn guess_exists(&self, word: &str) -> Result<bool, Error>;
}
