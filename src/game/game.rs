use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

use tokio::task::JoinHandle;
use tokio::sync::RwLock;

static TRIPLET_EXAMPLES: usize = 3;

#[derive(Debug)]
pub enum GameState {
    Config,
    Ongoing,
    Finished,
}

pub struct Round {
    pub substring: String,
}

// TODO: consider making fields private
pub struct Game {
    pub state: GameState,
    pub round_job: Option<JoinHandle<()>>,
    pub round: Option<Round>,
    triplets: HashMap<String, HashSet<String>>,
}

impl Game {
    pub fn new(words: &HashSet<String>) -> Game{
        Game{
            state: GameState::Config,
            round_job: None,
            round: None,
            triplets: Game::generate_triplets(words),
        }
    }

    // TODO: optimize this somehow cause rn !start_game takes 3-3.5 seconds to generate triplets
    // Goes over all words and saves at most N words for each triplet found.
    fn generate_triplets(words: &HashSet<String>) -> HashMap<String, HashSet<String>> {
        let start_t = SystemTime::now();

        let mut triplet_examples: HashMap<&str, HashSet<&str>> = HashMap::new();

        for w in words.iter() {
            for i in 3..=w.len() {
                let t = &w[i-3..i];

                if triplet_examples.contains_key(&t) {
                    let mut s = triplet_examples.get_mut(&t).unwrap();
                    if s.len() < TRIPLET_EXAMPLES { s.insert(&w); }
                } else {
                    let mut s: HashSet<&str> = HashSet::new();
                    s.insert(&w);
                    triplet_examples.insert(t, s);
                }
            }
        }

        let mut result: HashMap<String, HashSet<String>> = HashMap::new();
        for (t, s) in triplet_examples.iter() {
            if s.len() >= TRIPLET_EXAMPLES {
                let mut new_s = HashSet::<String>::new();
                for w in s.iter() { new_s.insert(w.to_string()); }
                result.insert(t.to_string(), new_s);
            }
        }

        let elapsed = start_t.elapsed().unwrap();

        println!("generated triplets");
        println!("total: {}", result.len());
        println!("time: {:?}", elapsed);

        result
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        match &self.round_job {
            Some(handle) => { handle.abort(); },
            None => {},
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_triplets() {
        let mut words: HashSet<String> = HashSet::new();
        words.insert(String::from("test"));
        words.insert(String::from("word"));
        words.insert(String::from("order"));
        words.insert(String::from("border"));

        let mut expected: HashMap<String, HashSet<String>> = HashMap::new();
        let mut v: HashSet<String> = HashSet::new();
        v.insert(String::from("word"));
        v.insert(String::from("order"));
        v.insert(String::from("border"));
        expected.insert(String::from("ord"), v);

        let result = Game::generate_triplets(&words);

        assert_eq!(result, expected);
    }

    #[test]
    fn generate_triplets_empty() {
        let mut words: HashSet<String> = HashSet::new();
        let result = Game::generate_triplets(&words);
        assert_eq!(result.len(), 0);
    }
}
