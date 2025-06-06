use crate::api::APIError;
use crate::schema::usersongs::{SongsResult, UsersongsRequest};
use crate::schema::{BeatMap, User};
use crate::util::data;
use crate::util::mongo::{MAPS_COLLECTION, USERS_COLLECTION};
use crate::util::warp::Replyable;
use anyhow::Error;
use mongodb::bson::doc;
use warp::{Rejection, Reply};

pub async fn usersongs(
    req: UsersongsRequest
) -> impl Responder {
    let user: User = data().await.database.query_one(USERS_COLLECTION, doc! { "id": req.user_id.to_string() })
        .await
        .map_err(|err| APIError::DatabaseError(err.into()))?
        .ok_or(APIError::KnownArgumentError(Error::msg("No user with that id")))?;
    
    let mut maps: Vec<BeatMap> = data().await.database.query(MAPS_COLLECTION, doc! { "charter_uid": user.id.to_string() })
        .await
        .map_err(|err| APIError::DatabaseError(err.into()))?;
    maps.sort_by(|first, second| first.upvotes.cmp(&second.upvotes).reverse());
    Ok(SongsResult {
        results: maps,
    })
}
