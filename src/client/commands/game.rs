use tokio::sync::Mutex;

use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;

use crate::client::{GameManagerValue, WordStorageValue};
use crate::game::{GameManager, GameCommand};

// Creates a new game and stores it in `ctx.data`.
// TODO: check for existing game first?
#[command]
#[aliases(new)]
pub async fn new_game(ctx: &Context, msg: &Message) -> CommandResult {
    {
        let mut data = ctx.data.write().await;
        let ws = data.get::<WordStorageValue>()
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
                        channel_id.say(&ctx_http, msg).await.unwrap();
                    },
                    None => return,
                }
            }
        });

    }
    msg.channel_id.say(&ctx.http, "New game created. Use \"!start_game\" to start it.").await?;
    Ok(())
}

// Starts a new game.
#[command]
#[aliases(start)]
pub async fn start_game(ctx: &Context, msg: &Message) -> CommandResult {
    {
        let mut data = ctx.data.write().await;
        match data.get_mut::<GameManagerValue>() {
            Some(game_ref) => {
                let mut game = game_ref.lock().await;
                if game.is_ongoing().await {
                    msg.channel_id.say(&ctx.http, "Game is already started.").await?;
                } else {
                    game.send_command(GameCommand::Start).await;
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
#[aliases(stop)]
pub async fn stop_game(ctx: &Context, msg: &Message) -> CommandResult {
    {
        let mut data = ctx.data.write().await;
        match data.get_mut::<GameManagerValue>() {
            Some(_) => {
                data.remove::<GameManagerValue>();
                msg.channel_id.say(&ctx.http, "Stopped current game.").await?;
            },
            None => {
                msg.channel_id.say(&ctx.http, "No game to stop.").await?;
            },
        }
    }
    Ok(())
}
