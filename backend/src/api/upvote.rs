use actix_web::{post, Responder};
use mongodb::bson::doc;
use crate::api::APIError;
use crate::schema::{BeatMap, User};
use crate::util::data;
use crate::util::mongo::{MAPS_COLLECTION, USERS_COLLECTION};
use crate::util::warp::Replyable;
use crate::schema::upvote::UpvoteRequest;

#[post("/api/upvote")]
pub async fn upvote(
    _req: UpvoteRequest,
    user: User,
    map: BeatMap,
) -> impl Responder {
    upvote_for_map(&map, &user).await?;
    Ok("Ok")
}

pub async fn upvote_for_map(map: &BeatMap, user: &User) -> Result<(), APIError> {
    if user.upvoted.contains(&map.id) {
        return Err(APIError::AlreadyUpvoted());
    }

    let database = data().await.database;
    database.update(MAPS_COLLECTION, doc! { "id": map.id.to_string() },
                    doc! {
                        "$inc": {
                            "upvotes": 1
                        }
                    }).await.map_err(APIError::database_error)?;
    database.update(USERS_COLLECTION, doc! { "id": user.id.to_string() },
                    doc! {
                        "$push": {
                            "upvoted": map.id.to_string()
                        }
                    }).await.map_err(APIError::database_error)?;
    Ok(())
}

#[post("/api/unvote")]
pub async fn unvote(
    _req: UpvoteRequest,
    mut user: User,
    map: BeatMap,
) -> impl Responder {
    unvote_for_map(&map, &mut user).await?;
    Ok("Ok")
}

pub async fn unvote_for_map(map: &BeatMap, user: &mut User) -> Result<(), APIError> {
    user.upvoted.remove(user.upvoted.iter().position(|elem| elem == &map.id).ok_or(APIError::AlreadyUpvoted())?);

    let database = data().await.database;
    database.update(MAPS_COLLECTION, doc! { "id": map.id.to_string() },
                    doc! {
                        "$inc": {
                            "upvotes": -1
                        }
                    }).await.map_err(APIError::database_error)?;
    database.update(USERS_COLLECTION, doc! { "id": user.id.to_string() },
                    doc! {
                        "$pull": {
                            "upvoted": map.id.to_string()
                        }
                    }).await.map_err(APIError::database_error)?;
    Ok(())
}
