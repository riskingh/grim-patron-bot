use std::error::Error;
use std::sync::{Arc, Mutex};

use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;

use crate::word_storage::WordStorage;

struct WordStorageValue;

impl TypeMapKey for WordStorageValue {
    type Value = Arc<Mutex<WordStorage>>;
}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "nope").await?;

    {
        let data = ctx.data.read().await;
        let ws_data = data
            .get::<WordStorageValue>()
            .expect("Word Storage is not initialized!")
            .clone();
        let ws = ws_data.lock().unwrap();
    }

    Ok(())
}

#[group]
#[commands(help)]
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
