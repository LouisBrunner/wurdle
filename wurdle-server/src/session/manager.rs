use super::session;
use super::traits;
use base64::{decode, encode};
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use itertools::Itertools;
use ring::{hmac, rand};
use std::io::prelude::*;

const SEPARATOR: &str = ".";

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
        let rng = rand::SystemRandom::new();
        let key = hmac::Key::generate(hmac::HMAC_SHA256, &rng)?;

        let serialized = session.serialize()?;
        let compressed = {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(serialized.as_bytes())?;
            encoder.finish()?
        };

        let tag = hmac::sign(&key, compressed.as_ref());

        let encoded = encode(compressed);
        let encoded_tag = encode(tag.as_ref());

        Ok(vec![encoded, encoded_tag].join(SEPARATOR))
    }

    pub fn deserialize(&self, payload: &str) -> Result<session::Session, traits::Error> {
        let rng = rand::SystemRandom::new();
        let key = hmac::Key::generate(hmac::HMAC_SHA256, &rng)?;

        let (encoded, encoded_tag) = payload
            .splitn(2, SEPARATOR)
            .collect_tuple()
            .ok_or(traits::Error::InvalidFormatting)?;

        let compressed = decode(encoded)?;
        let tag = decode(encoded_tag)?;
        hmac::verify(&key, compressed.as_ref(), tag.as_ref())?;

        let serialized = {
            let mut decoder = ZlibDecoder::new(Vec::new());
            decoder.write_all(compressed.as_ref())?;
            String::from_utf8(decoder.finish()?)?
        };
        let session = session::Session::deserialize(serialized.as_str())?;

        Ok(session)
    }
}
