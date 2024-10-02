use crate::search::SearchError;
use crate::search::SearchError::QueryError;
use firebase_auth::{FirebaseAuth, FirebaseUser};
use serde::Deserialize;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

#[derive(Debug, Deserialize)]
pub struct LoginArguments {
    pub firebase_token: String,
}

pub async fn login(query: &str, _db: Surreal<Client>, auth: FirebaseAuth) -> Result<(), SearchError> {
    let Ok(arguments) = serde_urlencoded::from_str::<LoginArguments>(query) else {
        return Err(QueryError());
    };

    let _user: FirebaseUser = auth.verify(&arguments.firebase_token).map_err(|_| SearchError::AuthError())?;
    Ok(())
}