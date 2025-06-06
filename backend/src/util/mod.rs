use crate::api::APIError;
use crate::schema::{AccountLink, User};
use crate::util::mongo::MongoDB;
use uuid::Uuid;

pub mod auth;
pub mod image;
pub mod mongo;

pub async fn get_user_from_link(account_link: AccountLink, database: &MongoDB) -> Result<User, APIError> {
    get_or_create_user(account_link.clone(), move || User {
        id: Uuid::new_v4(),
        links: vec![account_link.clone()],
        ..Default::default()
    }, database)
        .await
}

pub async fn get_or_create_user<F: Fn() -> User>(
    account_link: AccountLink,
    default_user: F,
    database: &MongoDB
) -> Result<User, APIError> {
    let user_opt = database
        .query_by_link(account_link)
        .await
        .map_err(APIError::database_error)?;
    Ok(user_opt.unwrap_or_else(default_user))
}