use super::traits;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
    InProgress { used_guesses: u8 },
    Failed,
    Won { used_guesses: u8 },
}

impl Status {
    pub fn to_string(&self) -> String {
        match self {
            Status::InProgress { .. } => "in_progress",
            Status::Failed => "failed",
            Status::Won { .. } => "guessed",
        }
        .to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub word_id: String,
    pub status: Status,
}

impl Session {
    pub fn new(word_id: &str) -> Self {
        Self {
            word_id: word_id.to_string(),
            status: Status::InProgress { used_guesses: 0 },
        }
    }

    pub fn serialize(&self) -> Result<String, traits::Error> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn deserialize(data: &str) -> Result<Self, traits::Error> {
        Ok(serde_json::from_str(data)?)
    }
}
