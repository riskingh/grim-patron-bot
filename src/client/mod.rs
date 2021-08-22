use std::error::Error;
use std::sync::Arc;

use serenity::client::{Client, EventHandler};
use serenity::framework::standard::macros::group;
use serenity::framework::standard::StandardFramework;
use serenity::model::user::User;
use serenity::prelude::*;
use tokio::sync::{Mutex, RwLock};

mod commands;
use crate::game::GameManager;
use crate::word_storage::WordStorage;
use commands::game::*;
use commands::help::*;

struct WordStorageValue;

impl TypeMapKey for WordStorageValue {
    type Value = Arc<RwLock<WordStorage>>;
}

struct GameManagerValue;

impl TypeMapKey for GameManagerValue {
    type Value = Mutex<GameManager<User>>;
}

#[group]
#[commands(help, new_game, start_game, stop_game, join_game)]
struct General;

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {}

pub async fn create_client(word_storage: WordStorage) -> Result<Client, Box<dyn Error>> {
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN env variable is required");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    let client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await?;

    {
        let mut data = client.data.write().await;
        data.insert::<WordStorageValue>(Arc::new(RwLock::new(word_storage)));
    }

    Ok(client)
}
