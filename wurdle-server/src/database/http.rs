use super::traits;
use std::collections::{HashMap, HashSet};
use std::io::Read;

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
    words_lookup: HashMap<String, usize>,
    guesses: HashSet<String>,
}

impl Database {
    pub fn new() -> Result<Self, traits::Error> {
        let words: Vec<String> = get_words_list(ANSWERS)?;
        let words_lookup = words
            .iter()
            .enumerate()
            .map(|(pos, word)| (word.clone(), pos))
            .collect();
        Ok(Self {
            words,
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

    fn word_length(&self) -> Result<usize, traits::Error> {
        Ok(self.words.len())
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
        Ok(self.guesses.contains(word))
    }
}
