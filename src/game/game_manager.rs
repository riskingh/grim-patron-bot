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
    command_tx: mpsc::Sender::<GameCommand>,
    command_job: JoinHandle<()>,
    message_tx: Arc<mpsc::Sender::<String>>,
}

impl GameManager {
    pub fn new(words: &HashSet<String>) -> (GameManager, mpsc::Receiver::<String>) {
        let game = Arc::new(RwLock::new(Game::new(words)));
        let game_clone = game.clone();

        // TODO: increase buffer maybe
        // TODO: add ACKs through one-time channels or rework it
        let (command_tx, command_rx) = mpsc::channel(1);
        let (message_tx, message_rx) = mpsc::channel(1000);
        let message_tx = Arc::new(message_tx);
        let message_tx_clone = message_tx.clone();

        (
            GameManager{
                game,
                command_tx,
                command_job: tokio::spawn(async move {
                    GameManager::command_job(command_rx, message_tx_clone, game_clone).await;
                }),
                message_tx,
            },
            message_rx,
        )
    }

    pub async fn is_ongoing(&self) -> bool {
        let game = self.game.read().await;
        matches!(game.state, GameState::Ongoing)
    }

    pub async fn send_command(&mut self, command: GameCommand) {
        println!("Sending command: {:?}", command);
        self.command_tx.send(command).await.unwrap();
    }

    // Handles client commands, mutates Game according to them.
    async fn command_job(
        mut command_rx: mpsc::Receiver<GameCommand>,
        mut message_tx: Arc<mpsc::Sender<String>>,
        game: Arc<RwLock<Game>>,
    ) {
        println!("Command job started!");
        loop {
            match command_rx.recv().await {
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
                            let message_tx_clone = message_tx.clone();
                            g.round_job = Some(tokio::spawn(async move {
                                GameManager::round_job(message_tx_clone, game_clone).await;
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
    async fn round_job(
        mut message_tx: Arc<mpsc::Sender<String>>,
        game: Arc<RwLock<Game>>,
    ) {
        message_tx.send(String::from("Round job started!")).await.unwrap();
        println!("Round job started!");
        let rounds = 3;
        let round_time = Duration::from_secs(5);
        for r in 1..=rounds {
            {
                let mut g = game.write().await;
                g.update_round();
                message_tx.send(
                    format!("Round #{}: **{}**.", r, g.round.as_ref().unwrap().triplet.to_uppercase())
                ).await.unwrap();
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

impl Drop for GameManager {
    fn drop(&mut self) {
        self.command_job.abort();
    }
}
