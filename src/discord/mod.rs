mod backlogger;

use crate::database::User;
use crate::ratelimiter::{Ratelimiter, UniqueIdentifier};
use crate::upload::{get_or_create_user, upload_beatmap, UploadError, UserId, MAX_SIZE};
use firebase_auth::FirebaseAuth;
use serenity::all::{Http, Ready};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use surrealdb::Surreal;
use tokio::time::timeout;
// Real server
//pub const WHITELISTED_GUILDS: [u64; 1] = [756193219737288836];
//pub const WHITELISTED_CHANNELS: [u64; 1] = [1244495595838640179];

// Testing server
pub const WHITELISTED_GUILDS: [u64; 0] = [];
pub const WHITELISTED_CHANNELS: [u64; 1] = [1298415906388574279];
#[derive(Clone)]
struct Handler {
    db: Surreal<surrealdb::engine::remote::ws::Client>,
    ratelimit: Arc<Mutex<Ratelimiter>>
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, message: Message) {
        if !WHITELISTED_CHANNELS.contains(&message.channel_id.into())  {
            return;
        }

        self.handle_message(&context.http, message).await;
    }

    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        //update_backlog(self, &ctx).await.unwrap();
    }
}

impl Handler {
    pub async fn handle_message(&self, _http: &Arc<Http>, message: Message) -> bool {
        for attachment in &message.attachments {
            if !attachment.filename.ends_with(".zip") && !attachment.filename.ends_with(".rar") {
                continue
            }

            if attachment.size > MAX_SIZE {
                //send_response(&http, &message, &"Failed to upload file! Size over 20MB limit!").await;
                continue
            }
            let file = attachment.download().await;
            match timeout(Duration::from_millis(5000), self.upload_map(file, message.author.id.into())).await {
                Ok(result) => match result {
                    Ok(_link) => {
                        /*send_response(&http, &message, &format!("Map uploaded! Try it at https://beatblockbrowser.me/search.html?{link}")).await;
                        if let Err(why) = message.react(&http, ReactionType::Unicode("🔼".to_string())).await {
                            println!("Error sending message: {why:?}");
                        }
                        if let Err(why) = message.react(&http, ReactionType::Unicode("🔽".to_string())).await {
                            println!("Error sending message: {why:?}");
                        }*/
                    },
                    Err(err) => {
                        //send_response(&context.http, &message, &format!("Failed to upload file! Error: {err}")).await;
                        println!("Upload error for {} ({}): {err:?}", message.link(), attachment.url);
                    }
                },
                Err(_) => {
                    //send_response(&context.http, &message, "Failed to read the zip file! Please report this for it to sync properly").await
                    println!("Timeout error for {}", message.link());
                }
            }
            return true;
        }
        false
    }
    
    pub async fn upload_map(&self, file: Result<Vec<u8>, serenity::Error>, user: u64) -> Result<String, UploadError> {
        let id = get_or_create_user(self.db.query(format!("SELECT id FROM users WHERE discord_id == {}", user))
                                      .await?.take::<Option<UserId>>(0)?, &self.db, User {
            discord_id: Some(user),
            ..Default::default()
        }).await?;
        upload_beatmap(file?, &self.db, &self.ratelimit, UniqueIdentifier::Discord(user), id).await
    }
}

pub async fn send_response(http: &Arc<Http>, message: &Message, error: &str) {
    /*match message.channel_id.send_message(&http, CreateMessage::new()
        .reference_message(message)
        .content(error)).await {
        Ok(message) => if let Err(why) = message.react(http, ReactionType::Unicode("❌".to_string())).await {
            println!("Error sending message: {why:?}")
        }
        Err(why) => println!("Error sending message: {why:?}")
    }*/
}

pub async fn run_bot(db: Surreal<surrealdb::engine::remote::ws::Client>, ratelimit: Arc<Mutex<Ratelimiter>>) {
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler {
            db,
            ratelimit
        }).await.expect("Error creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}