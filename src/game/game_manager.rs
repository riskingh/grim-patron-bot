use std::sync::Arc;
use std::time::Duration;
use std::collections::HashSet;

use tokio::sync::{mpsc, RwLock};

use tokio::time::sleep;
use tokio::task::JoinHandle;

use super::game::{Game, GameState, Round};

#[derive(Debug)]
pub enum GameCommand {
    Start,
    Stop,
}

pub struct GameManager {
    game: Arc<RwLock<Game>>,
    tx: mpsc::Sender::<GameCommand>,
    command_job: JoinHandle<()>,
}

impl GameManager {
    pub fn new(words: &HashSet<String>) -> GameManager {
        let game = Arc::new(RwLock::new(Game::new(words)));
        let (tx, rx) = mpsc::channel(1);
        let game_clone = game.clone();
        GameManager{
            game,
            tx,
            command_job: tokio::spawn(async move {
                GameManager::command_job(rx, game_clone).await;
            }),
        }
    }

    pub async fn is_ongoing(&self) -> bool {
        let game = self.game.read().await;
        matches!(game.state, GameState::Ongoing)
    }

    pub async fn send_command(&mut self, command: GameCommand) {
        println!("Sending command: {:?}", command);
        self.tx.send(command).await.unwrap();
    }

    // Handles client commands, mutates Game according to them.
    async fn command_job(mut rx: mpsc::Receiver<GameCommand>, game: Arc<RwLock<Game>>) {
        println!("Command job started!");
        loop {
            match rx.recv().await {
                Some(command) => {
                    println!("Command received: {:?}.", command);
                    match command {
                        GameCommand::Start => {
                            let mut g = game.write().await;
                            if !matches!(g.state, GameState::Config) {
                                println!("Game must be in `Config` state to be started.");
                                continue;
                            }
                            let game_clone = game.clone();
                            g.round_job = Some(tokio::spawn(async move {
                                GameManager::round_job(game_clone).await;
                            }));
                        },
                        _ => { todo!(); },
                    }
                },
                None => { todo!(); }
            }
        }
    }

    // Generates round data, handles timeouts.
    async fn round_job(game: Arc<RwLock<Game>>) {
        println!("Round job started!");
        let rounds = 3;
        let round_time = Duration::from_secs(5);
        for r in 1..=rounds {
            {
                let mut g = game.write().await;
                g.round = Some(
                    Round{substring: String::from("test123")},
                );
            }
            println!("Round: {}", r);
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

impl Drop for GameManager {
    fn drop(&mut self) {
        self.command_job.abort();
    }
}
