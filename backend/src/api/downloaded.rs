use crate::api::{get_map_request, APIError};
use crate::util::amazon::USERS_TABLE_NAME;
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use crate::SiteData;
use hyper::body::Incoming;
use hyper::Request;

pub async fn download(
    request: Request<Incoming>,
    identifier: UniqueIdentifier,
    data: &SiteData,
) -> Result<String, APIError> {
    let (map, user) =
        get_map_request(request, identifier, data, SiteAction::Download).await?;

    if user.downloaded.contains(&map.id) {
        return Err(APIError::AlreadyDownloaded());
    }
    data.amazon.add_to_list(USERS_TABLE_NAME, user.id.to_string(), "downloaded", map.id.to_string()).await?;
    Ok("Ok!".to_string())
}

pub async fn remove(
    request: Request<Incoming>,
    identifier: UniqueIdentifier,
    data: &SiteData,
) -> Result<String, APIError> {
    let (map, mut user) =
        get_map_request(request, identifier, data, SiteAction::Download).await?;
    user.downloaded.remove(user.downloaded.iter().position(|elem| elem == &map.id).ok_or(APIError::AlreadyUpvoted())?);
    data.amazon.overwrite_list(USERS_TABLE_NAME, user.id.to_string(), "downloaded", user.downloaded).await?;
    Ok("Ok!".to_string())
}
