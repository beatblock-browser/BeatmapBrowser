use std::sync::Arc;
use crate::discord::{Handler, WHITELISTED_CHANNELS, WHITELISTED_GUILDS};
use anyhow::Error;
use serenity::all::{ChannelId, Context, GuildId, Http, Message};
use serenity::futures::{stream, StreamExt};

pub async fn update_backlog(handler: &Handler, context: &Context) -> Result<(), Error> {
    for guild in WHITELISTED_GUILDS {
        for thread in GuildId::new(guild).get_active_threads(&context.http).await?.threads {
            let Some(parent) = thread.parent_id else {
                continue
            };
            if !WHITELISTED_CHANNELS.contains(&parent.into()) {
                continue
            }

            update_channel(thread.id, handler.clone(), context.http.clone()).await?;
        }
    }
    for channel in WHITELISTED_CHANNELS {
        update_channel(ChannelId::new(channel), handler.clone(), context.http.clone()).await?;
    }
    println!("Done updating backlog!");
    Ok(())
}

async fn update_channel(channel: ChannelId, handler: Handler, http: Arc<Http>) -> Result<(), Error> {
    let output: Vec<Result<Message, serenity::Error>> = channel.messages_iter(&http)
        .take_while(|message| {
            let value = message.as_ref()
                .map(|inner| {
                    !inner.reactions.iter().any(|reaction| reaction.me)
                })
                .map_err(|err| {
                    println!("HTTP Error: {}", err);
                    err
                })
                .unwrap_or(true);
            async move { value }
        })
        .collect::<Vec<_>>().await;
    if !stream::iter(output.into_iter().filter_map(Result::ok))
        .any(|message| handler.handle_message(&http, message)).await {
        println!("{}", channel);
    }
    Ok(())
}