use rand;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

static TRIPLET_EXAMPLES: usize = 3;

#[derive(Debug)]
pub enum GameError {
    PlayerKeyExistsError,
    GameMustBeInConfigStateError,
}

#[derive(Debug)]
pub enum GameState {
    Config,
    Ongoing,
    Finished,
}

pub struct Round {
    pub triplet: String,
    pub examples: HashSet<String>,
}

// TODO: consider making fields private
// Game object. Implements basic sync functions, requires manager to manage it's lifetime cycle.
pub struct Game<P> {
    pub state: GameState,
    pub round: Option<Round>,
    triplets: Vec<(String, HashSet<String>)>,
    players: HashMap<String, P>,
}

impl<P> Game<P> {
    pub fn new(words: &HashSet<String>) -> Game<P> {
        Game {
            state: GameState::Config,
            round: None,
            triplets: Game::<P>::generate_triplets(words),
            players: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, key: String, player: P) -> Result<(), GameError> {
        if !matches!(self.state, GameState::Config) {
            return Err(GameError::GameMustBeInConfigStateError);
        }

        if self.players.contains_key(&key) {
            Err(GameError::PlayerKeyExistsError)
        } else {
            self.players.insert(key, player);
            Ok(())
        }
    }

    pub fn update_round(&mut self) {
        let len = self.triplets.len();
        assert!(len > 1);
        let index = (rand::random::<f32>() * len as f32).floor() as usize;
        self.triplets.swap(index, len - 1);
        let round_triplet = self.triplets.pop().unwrap();
        self.round = Some(Round {
            triplet: round_triplet.0,
            examples: round_triplet.1,
        });
    }

    // TODO: optimize this somehow cause rn !start_game takes 3-3.5 seconds to generate triplets
    // Goes over all words and saves at most N words for each triplet found.
    fn generate_triplets(words: &HashSet<String>) -> Vec<(String, HashSet<String>)> {
        let start_t = SystemTime::now();

        let mut triplet_examples: HashMap<&str, HashSet<&str>> = HashMap::new();

        for w in words.iter() {
            for i in 3..=w.len() {
                let t = &w[i - 3..i];

                if triplet_examples.contains_key(&t) {
                    let mut s = triplet_examples.get_mut(&t).unwrap();
                    if s.len() < TRIPLET_EXAMPLES {
                        s.insert(&w);
                    }
                } else {
                    let mut s: HashSet<&str> = HashSet::new();
                    s.insert(&w);
                    triplet_examples.insert(t, s);
                }
            }
        }

        let mut result: Vec<(String, HashSet<String>)> = Vec::new();
        for (t, s) in triplet_examples.iter() {
            if s.len() >= TRIPLET_EXAMPLES {
                let mut new_s = HashSet::<String>::new();
                for w in s.iter() {
                    new_s.insert(w.to_string());
                }
                result.push((t.to_string(), new_s));
            }
        }

        let elapsed = start_t.elapsed().unwrap();

        println!("generated triplets");
        println!("total: {}", result.len());
        println!("time: {:?}", elapsed);

        result
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
