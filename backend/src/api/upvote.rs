use crate::api::{get_map_request, APIError};
use crate::util::amazon::{MAPS_TABLE_NAME, USERS_TABLE_NAME};
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use crate::SiteData;
use aws_sdk_dynamodb::types::AttributeValue;
use hyper::body::Incoming;
use hyper::Request;
use crate::util::database::{BeatMap, User};

pub async fn upvote(
    request: Request<Incoming>,
    identifier: UniqueIdentifier,
    data: &SiteData,
) -> Result<String, APIError> {
    let (map, user) = get_map_request(request, identifier, data, SiteAction::Search).await?;
    upvote_for_map(&map, &user, data).await?;
    Ok("Ok!".to_string())
}

pub async fn upvote_for_map(map: &BeatMap, user: &User, data: &SiteData) -> Result<(), APIError> {
    if user.upvoted.contains(&map.id) {
        return Err(APIError::AlreadyUpvoted());
    }

    data.amazon
        .update(MAPS_TABLE_NAME, map.id.to_string(), |builder| {
            builder
                .update_expression("SET upvotes = if_not_exists(upvotes, :start) + :inc")
                .expression_attribute_values(":start", AttributeValue::N("0".to_string()))
                .expression_attribute_values(":inc", AttributeValue::N("1".to_string()))
        })
        .await.map_err(APIError::database_error)?;
    data.amazon.add_to_list(USERS_TABLE_NAME, user.id.to_string(), "upvoted", map.id.to_string()).await?;
    Ok(())
}

pub async fn unvote(
    request: Request<Incoming>,
    identifier: UniqueIdentifier,
    data: &SiteData,
) -> Result<String, APIError> {
    let (map, mut user) =
        get_map_request(request, identifier, data, SiteAction::Search).await?;
    user.upvoted.remove(user.upvoted.iter().position(|elem| elem == &map.id).ok_or(APIError::AlreadyUpvoted())?);

    data.amazon
        .update(MAPS_TABLE_NAME, map.id.to_string(), |builder| {
            builder
                .update_expression("SET upvotes = :start - :dec")
                .expression_attribute_values(":start", AttributeValue::N(map.upvotes.to_string()))
                .expression_attribute_values(":dec", AttributeValue::N("1".to_string()))
        })
        .await.map_err(APIError::database_error)?;
    data.amazon.overwrite_list(USERS_TABLE_NAME, user.id.to_string(), "upvoted", user.upvoted).await?;
    Ok("Ok!".to_string())
}
