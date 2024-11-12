use crate::api::{get_map_request, APIError};
use crate::util::database::BeatMap;
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use crate::SiteData;
use hyper::body::Incoming;
use hyper::Request;

pub const ADMINS: [&'static str; 1] = ["gfde6dkqtey5trmfya8h"];

pub async fn delete(
    request: Request<Incoming>,
    identifier: UniqueIdentifier,
    data: &SiteData,
) -> Result<String, APIError> {
    let (map, user, map_id) =
        get_map_request(request, identifier, data, SiteAction::Search).await?;

    let user_id = user.id.unwrap().to_string();
    if map.charter_uid.unwrap_or(String::default()) != user_id &&
        !ADMINS.contains(&user_id.as_ref()) {
        return Err(APIError::PermissionError());
    }

    let _: Option<BeatMap> = data.db.delete(map_id).await
        .map_err(APIError::database_error)?;
    Ok("Song deleted!".to_string())
}
