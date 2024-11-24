use crate::api::APIError;
use crate::util::amazon::{Amazon, USERS_TABLE_NAME};
use crate::util::database::{AccountLink, User};
use anyhow::Error;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use std::sync::LockResult;
use uuid::Uuid;

pub mod amazon;
pub mod body;
pub mod database;
pub mod ratelimiter;

pub async fn collect_stream<S>(mut stream: S, max: usize) -> Result<Vec<u8>, Error>
where
    S: Stream<Item = Result<Bytes, hyper::Error>> + Unpin,
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

pub async fn get_user(account_link: AccountLink, amazon: &Amazon) -> Result<User, APIError> {
    get_or_create_user(
        account_link.clone(),
        amazon,
        move || User {
            id: Uuid::new_v4(),
            links: vec![account_link.clone()],
            ..Default::default()
        },
    )
    .await
}

pub async fn get_or_create_user<F: Fn() -> User>(
    account_link: AccountLink,
    amazon: &Amazon,
    default_user: F,
) -> Result<User, APIError> {
    if let Some(user) = amazon.query_by_link(account_link)
        .await
        .map_err(APIError::database_error)? {
        return Ok(user);
    }
    let user = default_user();
    amazon.upload(USERS_TABLE_NAME, &user, None::<&Vec<String>>)
        .await
        .map_err(APIError::database_error)?;
    Ok(user)
}

pub fn get_search_combos(input: &String) -> Vec<String> {
    let mut output = input.chars().take(6).fold(vec![], |mut acc, c| {
        if acc.is_empty() {
            acc.push(c.to_string());
        } else {
            acc.push(format!("{}{}", acc.last().unwrap(), c));
        }
        acc
    });
    output.push(input.clone());
    output
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
