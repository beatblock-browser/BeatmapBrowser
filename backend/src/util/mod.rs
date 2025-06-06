use crate::api::APIError;
use crate::util::mongo::MongoDB;
use crate::util::ratelimiter::Ratelimiter;
use crate::SiteData;
use firebase_auth::FirebaseAuth;
use lazy_static::lazy_static;
use std::sync::{Arc, LockResult};
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::schema::{AccountLink, User};

pub mod auth;
pub mod ratelimiter;
pub mod warp;
pub mod data;
pub mod image;
pub mod mongo;

static mut DATA: Option<SiteData> = None;
lazy_static! {
    static ref DATA_MUTEX: Mutex<()> = Mutex::new(());
}

pub async fn data() -> SiteData {
    unsafe {
        if DATA.is_none() {
            let _lock = match DATA_MUTEX.try_lock() {
                Ok(lock) => lock,
                Err(_) => {
                    let _ = DATA_MUTEX.lock().await;
                    return DATA.clone().unwrap();
                }
            };

            DATA = Some(SiteData {
                auth: FirebaseAuth::new("beatblockbrowser").await,
                database: MongoDB::connect().await.unwrap(),
                ratelimiter: Arc::new(std::sync::Mutex::new(Ratelimiter::new())),
            });
            return DATA.clone().unwrap();
        }
        DATA.clone().unwrap()
    }
}

pub async fn get_user_from_link(account_link: AccountLink) -> Result<User, APIError> {
    get_or_create_user(account_link.clone(), move || User {
        id: Uuid::new_v4(),
        links: vec![account_link.clone()],
        ..Default::default()
    })
        .await
}

pub async fn get_or_create_user<F: Fn() -> User>(
    account_link: AccountLink,
    default_user: F,
) -> Result<User, APIError> {
    let user_opt = data().await.database
        .query_by_link(account_link)
        .await
        .map_err(APIError::database_error)?;
    Ok(user_opt.unwrap_or_else(default_user))
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
