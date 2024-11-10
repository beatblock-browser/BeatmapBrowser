mod backlogger;

use crate::api::search::SearchArguments;
use crate::api::upload::{upload_beatmap, MAX_SIZE};
use crate::api::APIError;
use crate::discord::backlogger::update_backlog;
use crate::util::ratelimiter::{Ratelimiter, UniqueIdentifier};
use crate::util::{get_user, LockResultExt};
use serenity::all::{Http, Ready, UserId};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::collections::HashSet;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use surrealdb::opt::PatchOp;
use surrealdb::Surreal;
use tokio::time::timeout;
use crate::util::database::{BeatMap, User};

// Real server
//pub const WHITELISTED_GUILDS: [u64; 1] = [756193219737288836];
//pub const WHITELISTED_CHANNELS: [u64; 1] = [1244495595838640179];

// Testing server
pub const WHITELISTED_GUILDS: [u64; 0] = [];
pub const WHITELISTED_CHANNELS: [u64; 1] = [1298415906388574279];

#[derive(Clone)]
struct Handler {
    db: Surreal<surrealdb::engine::remote::ws::Client>,
    ratelimit: Arc<Mutex<Ratelimiter>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, message: Message) {
        if let Some(parent) = message
            .channel_id
            .to_channel(&context.http)
            .await
            .unwrap()
            .guild()
            .unwrap()
            .parent_id
        {
            if !WHITELISTED_CHANNELS.contains(&parent.into()) {
                return;
            }
        } else if !WHITELISTED_CHANNELS.contains(&message.channel_id.into()) {
            return;
        }

        self.handle_message(&context.http, message, &HashSet::default())
            .await;
    }

    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        update_backlog(self, &ctx).await.unwrap();
    }
}

impl Handler {
    pub async fn handle_message(
        &self,
        http: &Arc<Http>,
        message: Message,
        upvotes: &HashSet<UserId>,
    ) -> bool {
        let mut found = false;
        for attachment in &message.attachments {
            if !attachment.filename.ends_with(".zip") && !attachment.filename.ends_with(".rar") {
                continue;
            }

            if attachment.size > MAX_SIZE {
                println!(
                    "Skipped massive file {}: {}",
                    attachment.filename, attachment.url
                );
                send_response(&http, &message, &"Failed to upload file! Size over 20MB limit!").await;
                continue;
            }

            let file = attachment.download().await;
            match timeout(
                Duration::from_millis(5000),
                self.upload_map(file, message.author.id.into(), upvotes.clone()),
            )
                .await
            {
                Ok(result) => match result {
                    Ok(link) => {
                        found = true;
                        send_response(&http, &message, &format!("Map uploaded! Try it at https://beatblockbrowser.me/search.html?{link}")).await;
                        /*if let Err(why) = message.react(&http, ReactionType::Unicode("🔼".to_string())).await {
                            println!("Error sending message: {why:?}");
                        }
                        if let Err(why) = message.react(&http, ReactionType::Unicode("🔽".to_string())).await {
                            println!("Error sending message: {why:?}");
                        }*/
                    }
                    Err(err) => {
                        send_response(&http, &message, &format!("Failed to upload file! Error: {err}")).await;
                        println!(
                            "Upload error for {} ({}): {err:?}",
                            message.link(),
                            attachment.url
                        );
                    }
                },
                Err(_) => {
                    send_response(&http, &message, "Failed to read the zip file! Please report this for it to sync properly").await;
                    println!("Timeout error for {}", message.link());
                }
            }
        }
        found
    }

    pub async fn upload_map(
        &self,
        file: Result<Vec<u8>, serenity::Error>,
        user_id: u64,
        upvotes: HashSet<UserId>,
    ) -> Result<String, APIError> {
        let user = get_user(false, user_id.to_string(), &self.db).await?;
        self.ratelimit.lock().ignore_poison().clear();
        let map = upload_beatmap(
            file?,
            &self.db,
            &self.ratelimit,
            UniqueIdentifier::Discord(user_id),
            user.id.unwrap(),
        )
            .await?;
        if map.upvotes == 0 {
            for user in &upvotes {
                let user = get_user(false, user.to_string(), &self.db).await?;
                let user_id = user.id.as_ref().unwrap();
                let _: Option<User> = self.db.update(("users".to_string(), user_id.id.to_string()))
                    .patch(PatchOp::add("upvoted", map.id.clone().unwrap())).await
                    .map_err(APIError::database_error)?;
            }
            let map_id = map.id.as_ref().unwrap();
            let _: Option<BeatMap> = self.db.update(("beatmaps".to_string(), map_id.id.to_string()))
                .patch(PatchOp::replace("upvotes", upvotes.len())).await
                .map_err(APIError::database_error)?;
        }
        serde_urlencoded::to_string(&SearchArguments {
            query: map.song.clone(),
        })
            .map_err(|err| APIError::SongNameError(err))
    }
}

pub async fn send_response(_http: &Arc<Http>, _message: &Message, _error: &str) {
    /*match message.channel_id.send_message(&http, CreateMessage::new()
        .reference_message(message)
        .content(error)).await {
        Ok(message) => if let Err(why) = message.react(http, ReactionType::Unicode("❌".to_string())).await {
            println!("Error sending message: {why:?}")
        }
        Err(why) => println!("Error sending message: {why:?}")
    }*/
}

pub async fn run_bot(
    db: Surreal<surrealdb::engine::remote::ws::Client>,
    ratelimit: Arc<Mutex<Ratelimiter>>,
) {
    // Login with a bot token from the environment
    let Some(token) = env::args().nth(2) else {
        println!("No token provided, not running the bot");
        return;
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { db, ratelimit })
        .await
        .expect("Error creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
