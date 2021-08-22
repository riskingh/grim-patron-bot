use std::future::Future;

use tokio::sync::{Mutex, MutexGuard};
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::model::user::User;
use serenity::utils::MessageBuilder;

use crate::client::{GameManagerValue, WordStorageValue};
use crate::game::{GameManager, GameManagerError};

// Creates a new game and stores it in `ctx.data`.
// TODO: check for existing game first?
#[command]
#[aliases(new)]
pub async fn new_game(ctx: &Context, msg: &Message) -> CommandResult {
    {
        let mut data = ctx.data.write().await;
        let ws = data
            .get::<WordStorageValue>()
            .expect("WordStorage must be initialized.")
            .clone();
        let ws_read = ws.read().await;
        let (game, mut message_rx) = GameManager::new(&ws_read.words);
        data.insert::<GameManagerValue>(Mutex::new(game));

        let channel_id = msg.channel_id;
        let ctx_http = ctx.http.clone();
        tokio::spawn(async move {
            loop {
                match message_rx.recv().await {
                    Some(msg) => {
                        channel_id.say(&ctx_http, &msg.text).await.unwrap();
                        msg.ack().unwrap();
                    }
                    None => return,
                }
            }
        });
    }
    msg.channel_id
        .say(
            &ctx.http,
            "New game created. Use \"!start_game\" to start it.",
        )
        .await?;
    Ok(())
}


// Starts created game.
#[command]
#[aliases(start)]
pub async fn start_game(ctx: &Context, msg: &Message) -> CommandResult {
    let response = {
        let mut data = ctx.data.write().await;
        match data.get_mut::<GameManagerValue>() {
            Some(gm_ref) => {
                let mut gm = gm_ref.lock().await;
                match gm.start_game().await {
                    Ok(_) => "Game is started!",
                    Err(e) => {
                        match *e {
                            GameManagerError::GameAlreadyStartedError => "Game is already started!",
                            _ => panic!("Unhandled error: {:?}", e)
                        }
                    }
                }
            },
            None => "You must create a game first with \"!new_game\" command.",
        }
    };
    msg.channel_id.say(&ctx.http, response).await?;
    Ok(())
}

#[command]
#[aliases(stop)]
pub async fn stop_game(ctx: &Context, msg: &Message) -> CommandResult {
    {
        let mut data = ctx.data.write().await;
        match data.get_mut::<GameManagerValue>() {
            Some(_) => {
                data.remove::<GameManagerValue>();
                msg.channel_id
                    .say(&ctx.http, "Stopped current game.")
                    .await?;
            }
            None => {
                msg.channel_id.say(&ctx.http, "No game to stop.").await?;
            }
        }
    }
    Ok(())
}

#[command]
#[aliases(join)]
pub async fn join_game(ctx: &Context, msg: &Message) -> CommandResult {
    {
        let mut data = ctx.data.write().await;
        match data.get_mut::<GameManagerValue>() {
            Some(game_ref) => {
                let mut game = game_ref.lock().await;
                game.add_player(msg.author.name.clone(), msg.author.clone())
                    .await;
            }
            None => {
                todo!()
            }
        }
    }
    Ok(())
}
