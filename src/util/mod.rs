use std::fmt::{Debug, Display};
use std::sync::LockResult;
use anyhow::Error;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use hyper::StatusCode;

pub mod body;
pub mod database;
pub mod ratelimiter;

pub trait WebError: Debug + Display {
    fn get_code(&self) -> StatusCode;
}

pub fn to_weberr<T, E: WebError + 'static>(result: Result<T, E>) -> Result<T, Box<dyn WebError>> {
    result.map_err(|err| Box::new(err) as Box<dyn WebError>)
}

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