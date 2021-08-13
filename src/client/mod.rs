use std::error::Error;
use std::sync::Arc;

use serenity::client::{Client, EventHandler};
use serenity::framework::standard::macros::group;
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::prelude::*;
use tokio::sync::Mutex;

mod commands;
use crate::word_storage::WordStorage;
use crate::game::Game;
use commands::help::*;
use commands::game::*;

struct WordStorageValue;

impl TypeMapKey for WordStorageValue {
    type Value = Arc<Mutex<WordStorage>>;
}

struct GameValue;

impl TypeMapKey for GameValue {
    type Value = Mutex<Game>;
}

#[group]
#[commands(help,new_game,start_game,stop_game)]
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
        data.insert::<WordStorageValue>(Arc::new(Mutex::new(word_storage)));
    }

    Ok(client)
}
