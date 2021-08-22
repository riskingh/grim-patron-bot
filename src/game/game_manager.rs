use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use std::error;
use std::fmt;

use tokio::sync::{mpsc, RwLock};

use tokio::task::JoinHandle;
use tokio::time::sleep;

use super::game::{Game, GameState, Round};
use super::message_channel;

#[derive(Debug)]
pub enum GameManagerError {
    GameAlreadyStartedError,
    GameAlreadyFinishedError,
}

impl fmt::Display for GameManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GameManagerError::GameAlreadyStartedError => write!(f, "game is already started"),
            GameManagerError::GameAlreadyFinishedError => write!(f, "game is already finished"),
        }
    }
}

impl error::Error for GameManagerError {}

type GameManagerResult = Result<(), GameManagerError>;

// GameManager manages Game object by spawning tokio async jobs and handling commands from client.
// Is aware of async runtime, but not aware of exact client implementation.
pub struct GameManager<P> {
    game: Arc<RwLock<Game<P>>>,
    message_tx: Arc<message_channel::MessageSender>,
    round_job: Option<JoinHandle<()>>,
}

impl<P> GameManager<P>
where
    P: std::marker::Sync + std::marker::Send + std::fmt::Debug + 'static,
{
    pub fn new(words: &HashSet<String>) -> (GameManager<P>, message_channel::MessageReceiver) {
        let game = Arc::new(RwLock::new(Game::new(words)));
        let game_clone = game.clone();

        let (message_tx, message_rx) = message_channel::new(1);
        let message_tx = Arc::new(message_tx);
        let message_tx_clone = message_tx.clone();

        (
            GameManager { game, message_tx, round_job: None },
            message_rx,
        )
    }

    // Adds a player to the game. Game must be in `Config` state.
    pub async fn add_player(&mut self, key: String, player: P) -> GameManagerResult {
        let mut g = self.game.write().await;
        match g.add_player(key.clone(), player) {
            Ok(_) => {
                self.message_tx
                    .send_and_wait(format!("Player \"{}\" joined!", key))
                    .await;
            }
            Err(e) => {
                self.message_tx
                    .send_and_wait(format!("Cannot add player \"{}\": {:?}", key, e))
                    .await;
            }
        }
        Ok(())  // TODO: fix it
    }

    // Spawns round job, moves game into `Ongoing` state.
    pub async fn start_game(&mut self) -> GameManagerResult {
        let mut g = self.game.write().await;
        if !matches!(g.state, GameState::Config) {
            return Err(GameManagerError::GameAlreadyStartedError);
        }
        let game_clone = self.game.clone();
        let message_tx_clone = self.message_tx.clone();
        self.round_job = Some(tokio::spawn(async move {
            GameManager::round_job(message_tx_clone, game_clone).await;
        }));
        g.state = GameState::Ongoing;
        Ok(())
    }

    // Stops all jobs and moves game into `Finished` state.
    pub async fn stop_game(&mut self) -> GameManagerResult {
        let mut g = self.game.write().await;
        if matches!(g.state, GameState::Finished) {
            return Err(GameManagerError::GameAlreadyFinishedError);
        }
        todo!();
        // ....
    }

    // Generates round data, handles timeouts.
    async fn round_job(
        mut message_tx: Arc<message_channel::MessageSender>,
        game: Arc<RwLock<Game<P>>>,
    ) {
        message_tx
            .send_and_wait(String::from("Round job started!"))
            .await;
        println!("Round job started!");
        let rounds = 3;
        let round_time = Duration::from_secs(5);
        for r in 1..=rounds {
            {
                let mut g = game.write().await;
                g.update_round();
                message_tx
                    .send_and_wait(format!(
                        "Round #{}: **{}**.",
                        r,
                        g.round.as_ref().unwrap().triplet.to_uppercase()
                    ))
                    .await;
            }
            sleep(round_time).await;
        }
        println!("Game is finishing...");
        {
            let mut g = game.write().await;
            g.state = GameState::Finished;
        }
        println!("Game is finished.");
    }
}

impl<P> Drop for GameManager<P> {
    fn drop(&mut self) {
        match &self.round_job {
            None => {},
            Some(job) => job.abort(),
        }
    }
}
