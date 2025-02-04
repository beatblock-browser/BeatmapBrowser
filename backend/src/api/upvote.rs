use mongodb::bson::doc;
use crate::api::APIError;
use crate::util::database::{BeatMap, User};
use warp::{Rejection, Reply};
use crate::util::data;
use crate::util::mongo::{MAPS_COLLECTION, USERS_COLLECTION};
use crate::util::warp::Replyable;

pub async fn upvote(
    user: User,
    map: BeatMap,
) -> Result<impl Reply, Rejection> {
    upvote_for_map(&map, &user).await?;
    Ok("Ok".reply())
}

pub async fn upvote_for_map(map: &BeatMap, user: &User) -> Result<(), APIError> {
    if user.upvoted.contains(&map.id) {
        return Err(APIError::AlreadyUpvoted());
    }

    let database = data().await.database;
    database.update(MAPS_COLLECTION, doc! { "id": map.id },
                    doc! {
                        "$inc": {
                            "upvotes": 1
                        }
                    }).await?;
    database.update(USERS_COLLECTION, doc! { "id": user.id },
                    doc! {
                        "$append": {
                            "upvoted": map.id
                        }
                    }).await?;
    Ok(())
}

pub async fn unvote(
    mut user: User,
    map: BeatMap,
) -> Result<impl Reply, Rejection> {
    unvote_for_map(&map, &mut user).await?;
    Ok("Ok".reply())
}

pub async fn unvote_for_map(map: &BeatMap, user: &mut User) -> Result<(), APIError> {
    user.upvoted.remove(user.upvoted.iter().position(|elem| elem == &map.id).ok_or(APIError::AlreadyUpvoted())?);

    let database = data().await.database;
    database.update(MAPS_COLLECTION, doc! { "id": map.id },
                    doc! {
                        "$inc": {
                            "upvotes": -1
                        }
                    }).await?;
    database.update(USERS_COLLECTION, doc! { "id": user.id },
                    doc! {
                        "$pull": {
                            "upvoted": map.id
                        }
                    }).await?;
    Ok(())
}
