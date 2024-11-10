use crate::api::{get_map_request, APIError};
use crate::util::database::{BeatMap, User};
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use crate::SiteData;
use hyper::body::Incoming;
use hyper::Request;
use surrealdb::opt::PatchOp;
use surrealdb::sql::Thing;

pub async fn upvote(
    request: Request<Incoming>,
    identifier: UniqueIdentifier,
    data: &SiteData,
) -> Result<String, APIError> {
    let (map, user, map_id) =
        get_map_request(request, identifier, data, SiteAction::Search).await?;

    if user.upvoted.contains(map.id.as_ref().unwrap()) {
        return Err(APIError::AlreadyUpvoted());
    }

    let _: Option<BeatMap> = data
        .db
        .update(map_id.clone())
        .patch(PatchOp::replace("upvotes", map.upvotes + 1))
        .await
        .map_err(APIError::database_error)?;
    let _: Option<User> = data
        .db
        .update(("users", user.id.unwrap().id.to_string()))
        .patch(PatchOp::add("upvoted", Thing::from(map_id)))
        .await
        .map_err(APIError::database_error)?;

    Ok("Ok!".to_string())
}

pub async fn unvote(
    request: Request<Incoming>,
    identifier: UniqueIdentifier,
    data: &SiteData,
) -> Result<String, APIError> {
    let (map, user, map_id) =
        get_map_request(request, identifier, data, SiteAction::Search).await?;

    let _: Option<BeatMap> = data
        .db
        .update(map_id)
        .patch(PatchOp::replace("upvotes", map.upvotes - 1))
        .await
        .map_err(APIError::database_error)?;
    let _: Option<User> = data
        .db
        .update(("users", user.id.unwrap().id.to_string()))
        .patch(PatchOp::remove(&*format!(
            "upvoted/{}",
            user.upvoted
                .iter()
                .position(|e| e == map.id.as_ref().unwrap())
                .ok_or(APIError::AlreadyUpvoted())?
        )))
        .await
        .map_err(APIError::database_error)?;

    Ok("Ok!".to_string())
}
