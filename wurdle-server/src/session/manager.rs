use super::session;
use super::traits;

#[derive(Clone)]
pub struct SessionManager {
    token: String,
}

impl SessionManager {
    pub fn new(token: &str) -> Result<Self, traits::Error> {
        Ok(Self {
            token: token.to_string(),
        })
    }

    pub fn serialize(&self, session: session::Session) -> Result<String, traits::Error> {
        Ok(session.serialize()?)
    }

    pub fn deserialize(&self, data: &str) -> Result<session::Session, traits::Error> {
        Ok(session::Session::deserialize(data)?)
    }
}
