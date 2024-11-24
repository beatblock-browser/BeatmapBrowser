use crate::api::{get_map_request, APIError};
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use crate::SiteData;
use hyper::body::Incoming;
use hyper::Request;
use crate::util::amazon::{MAPS_TABLE_NAME, USERS_TABLE_NAME};

pub const ADMINS: [&'static str; 1] = ["gfde6dkqtey5trmfya8h"];

pub async fn delete(
    request: Request<Incoming>,
    identifier: UniqueIdentifier,
    data: &SiteData,
) -> Result<String, APIError> {
    let (map, mut user) =
        get_map_request(request, identifier, data, SiteAction::Search).await?;

    if map.charter_uid != user.id && !ADMINS.contains(&user.id.to_string().as_str()) {
        return Err(APIError::PermissionError());
    }
    
    user.maps.remove(user.maps.iter().position(|elem| elem == &map.id).ok_or(APIError::AlreadyDownloaded())?);
    data.amazon.remove(MAPS_TABLE_NAME, "id", map.id.to_string()).await
        .map_err(APIError::database_error)?;
    data.amazon.overwrite_list(USERS_TABLE_NAME, user.id.to_string(), "maps", user.maps).await?;
    data.amazon.delete_object(format!("{}.zip", map.id).as_str()).await.map_err(APIError::database_error)?;
    data.amazon.delete_object(format!("{}.png", map.id).as_str()).await.map_err(APIError::database_error)?;
    Ok("Song deleted!".to_string())
}
