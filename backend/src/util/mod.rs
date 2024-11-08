use anyhow::Error;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use std::sync::LockResult;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use crate::api::APIError;
use crate::util::database::User;

pub mod body;
pub mod database;
pub mod ratelimiter;

pub async fn collect_stream<S>(mut stream: S, max: usize) -> Result<Vec<u8>, Error>
where
    S: Stream<Item=Result<Bytes, hyper::Error>> + Unpin,
{
    let mut collected = Vec::with_capacity(max);
    let mut total = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let chunk_len = chunk.len();

        if total + chunk_len > max {
            return Err(Error::msg("Size limit exceeded"));
        } else {
            collected.extend_from_slice(&chunk);
            total += chunk_len;
        }
    }

    Ok(collected)
}

pub async fn get_user(google: bool, id: String, db: &Surreal<Client>) -> Result<User, APIError> {
    let default_user = if google {
        User {
            google_id: Some(id.clone()),
            ..Default::default()
        }
    } else {
        User {
            discord_id: Some(id.parse().unwrap()),
            ..Default::default()
        }
    };
    let checking = if google {
        format!("google_id == '{}'", id)
    } else {
        format!("discord_id == {}", id)
    };
    get_or_create_user(
        format!("SELECT * FROM users WHERE {}", checking), db, default_user).await
}


pub async fn get_or_create_user(query: String, db: &Surreal<Client>, default_user: User) -> Result<User, APIError> {
    Ok(if let Some(id) = db.query(query).await.map_err(APIError::database_error)?
        .take::<Option<User>>(0).map_err(APIError::database_error)? {
        id
    } else {
        let Some(user): Option<User> = db.create("users").content(default_user)
            .await.map_err(APIError::database_error)? else {
            return Err(APIError::UnknownDatabaseError("Failed to create a user in the users database".to_string()));
        };
        user
    })
}

pub fn get_beatmap_id(thing: &Thing) -> (String, String) {
    (thing.tb.clone(), thing.id.to_string()[3..thing.id.to_string().len() - 3].to_string()).into()
}

pub trait LockResultExt {
    type Guard;

    /// Returns the lock guard even if the mutex is [poisoned].
    ///
    /// [poisoned]: https://doc.rust-lang.org/stable/std/sync/struct.Mutex.html#poisoning
    fn ignore_poison(self) -> Self::Guard;
}

impl<Guard> LockResultExt for LockResult<Guard> {
    type Guard = Guard;

    fn ignore_poison(self) -> Guard {
        self.unwrap_or_else(|e| e.into_inner())
    }
}