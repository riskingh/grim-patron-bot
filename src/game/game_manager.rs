use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, RwLock};

use tokio::task::JoinHandle;
use tokio::time::sleep;

use super::game::{Game, GameState, Round};
use super::message_channel;

#[derive(Debug)]
pub enum GameCommand<P> {
    Start,
    Stop,
    AddPlayer(String, P),
}

pub struct GameManager<P> {
    game: Arc<RwLock<Game<P>>>,
    command_tx: mpsc::Sender<GameCommand<P>>,
    command_job: JoinHandle<()>,
    message_tx: Arc<message_channel::MessageSender>,
}

impl<P> GameManager<P>
where
    P: std::marker::Sync + std::marker::Send + std::fmt::Debug + 'static,
{
    pub fn new(words: &HashSet<String>) -> (GameManager<P>, message_channel::MessageReceiver) {
        let game = Arc::new(RwLock::new(Game::new(words)));
        let game_clone = game.clone();

        // TODO: increase buffer maybe
        // TODO: add ACKs through one-time channels or rework it
        let (command_tx, command_rx) = mpsc::channel(1);
        let (message_tx, message_rx) = message_channel::new(1);
        let message_tx = Arc::new(message_tx);
        let message_tx_clone = message_tx.clone();

        (
            GameManager {
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

    // TODO: remove command channel and invoke methods instead
    pub async fn send_command(&mut self, command: GameCommand<P>) {
        // TODO: handle error properly
        self.command_tx.send(command).await.unwrap();
    }

    pub async fn add_player(&mut self, key: String, player: P) {
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
    }

    // Handles client commands, mutates Game according to them.
    async fn command_job(
        mut command_rx: mpsc::Receiver<GameCommand<P>>,
        mut message_tx: Arc<message_channel::MessageSender>,
        game: Arc<RwLock<Game<P>>>,
    ) {
        println!("Command job started!");
        loop {
            match command_rx.recv().await {
                Some(command) => match command {
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
                    }
                    _ => {
                        todo!();
                    }
                },
                None => {
                    todo!();
                }
            }
        }
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
        self.command_job.abort();
    }
}
