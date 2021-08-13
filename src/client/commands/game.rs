use std::sync::Arc;
use tokio::sync::Mutex;

use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;

use crate::client::{GameValue, WordStorageValue};
use crate::game::Game;

#[command]
pub async fn new_game(ctx: &Context, msg: &Message) -> CommandResult {
    {
        let game = Game::new();
        let mut data = ctx.data.write().await;
        data.insert::<GameValue>(Arc::new(Mutex::new(game)));
    }
    msg.channel_id.say(&ctx.http, "New game created. Use \"!start_game\" to start it.").await?;

    Ok(())
}

#[command]
pub async fn start_game(ctx: &Context, msg: &Message) -> CommandResult {
    {
        let mut data = ctx.data.write().await;
        match data.get_mut::<GameValue>() {
            Some(game) => {
                let mut game = game.clone();
                let mut x = game.lock().await;
                if x.is_started() {
                    msg.channel_id.say(&ctx.http, "Game is already started.").await?;
                } else {
                    x.start();
                    msg.channel_id.say(&ctx.http, "Game is started!").await?;
                }
            },
            None => {
                msg.channel_id.say(
                    &ctx.http,
                    "You must create a game first with \"!new_game\" command.",
                ).await?;
            },
        }
    }
    Ok(())
}

#[command]
pub async fn stop_game(ctx: &Context, msg: &Message) -> CommandResult {
    {
        let mut data = ctx.data.write().await;
        match data.get_mut::<GameValue>() {
            Some(game) => {
                data.remove::<GameValue>();
                msg.channel_id.say(&ctx.http, "Stopped current game.").await?;
            },
            None => {
                msg.channel_id.say(&ctx.http, "No game to stop.").await?;
            },
        }
    }
    Ok(())
}
