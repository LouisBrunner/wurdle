use super::traits;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::collections::{HashMap, HashSet};
use std::io::Read;

const RESTRICTED_LENGTH: usize = 500;
static SAMPLING_SEED: &str = "sample me baby";
static ALLOWED_WORDS: &str = "https://gist.githubusercontent.com/cfreshman/40608e78e83eb4e1d60b285eb7e9732f/raw/2f51b4f2bb96c02e1dee37808b2eed4ef23a3150/wordle-nyt-allowed-guesses.txt";
static ANSWERS: &str = "https://gist.githubusercontent.com/cfreshman/a7b776506c73284511034e63af1017ee/raw/845966807347a7b857d53294525263408be967ce/wordle-nyt-answers-alphabetical.txt";

fn do_request(url: &str) -> Result<String, traits::Error> {
    let mut res = reqwest::blocking::get(url)?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;
    Ok(body)
}

fn get_words_list<T: FromIterator<String>>(url: &str) -> Result<T, traits::Error> {
    let body = do_request(url)?;
    Ok(body.split('\n').map(str::to_string).collect::<T>())
}

#[derive(Clone)]
pub struct Database {
    words: Vec<String>,
    restricted_words: Vec<String>,
    words_lookup: HashMap<String, usize>,
    guesses: HashSet<String>,
}

fn get_rng() -> StdRng {
    let mut seed: [u8; 32] = [0; 32];
    seed[..SAMPLING_SEED.len()].copy_from_slice(SAMPLING_SEED.as_bytes());
    StdRng::from_seed(seed)
}

impl Database {
    pub fn new() -> Result<Self, traits::Error> {
        let mut words: Vec<String> = get_words_list(ANSWERS)?;
        words.sort();
        let words_lookup = words
            .iter()
            .enumerate()
            .map(|(pos, word)| (word.clone(), pos))
            .collect();

        let mut restricted_words: Vec<String> = words.iter().map(|word| word.clone()).collect();
        let mut rng = get_rng();
        restricted_words.shuffle(&mut rng);
        restricted_words.truncate(RESTRICTED_LENGTH);
        restricted_words.sort();

        Ok(Self {
            words,
            restricted_words,
            words_lookup,
            guesses: get_words_list(ALLOWED_WORDS)?,
        })
    }
}

impl traits::Database for Database {
    fn word_for_id(&self, id: &str) -> Result<traits::Word, traits::Error> {
        let index = id
            .parse::<usize>()
            .map_err(|_e| traits::Error::InvalidID { id: id.to_string() })?;
        self.word_for_index(index)
    }

    fn word_exists(&self, word: &str) -> Result<traits::Word, traits::Error> {
        match self.words_lookup.get(&word.to_string()) {
            Some(index) => Ok(traits::Word {
                word_id: index.to_string(),
                word: word.to_string(),
            }),
            None => Err(traits::Error::MissingWord {
                word: word.to_string(),
            }),
        }
    }

    fn word_length(&self, restricted: bool) -> Result<usize, traits::Error> {
        Ok(match restricted {
            true => &self.restricted_words,
            false => &self.words,
        }
        .len())
    }

    fn word_for_index(&self, index: usize) -> Result<traits::Word, traits::Error> {
        let len = self.words.len();
        if len <= index {
            return Err(traits::Error::OutOfBounds {
                index,
                maximum: len,
            });
        }
        Ok(traits::Word {
            word_id: index.to_string(),
            word: self.words[index].clone(),
        })
    }

    fn guess_exists(&self, word: &str) -> Result<bool, traits::Error> {
        Ok(self.guesses.contains(word) || self.words_lookup.contains_key(word))
    }

    fn allowed_words(&self) -> Result<Vec<String>, traits::Error> {
        return Ok(Vec::from_iter(self.guesses.clone()));
    }

    fn answer_words(&self, restricted: bool) -> Result<Vec<String>, traits::Error> {
        Ok(match restricted {
            true => &self.restricted_words,
            false => &self.words,
        }
        .to_vec())
    }
}
