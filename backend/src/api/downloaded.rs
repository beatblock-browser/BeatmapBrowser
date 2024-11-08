use crate::api::{get_map_request, APIError};
use crate::util::database::User;
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use crate::SiteData;
use hyper::body::Incoming;
use hyper::Request;
use surrealdb::opt::PatchOp;
use surrealdb::sql::Thing;

pub async fn download(request: Request<Incoming>, identifier: UniqueIdentifier, data: &SiteData) -> Result<String, APIError> {
    let (map, user, map_id) = get_map_request(request, identifier, data, SiteAction::Download).await?;

    if user.downloaded.contains(map.id.as_ref().unwrap()) {
        return Err(APIError::AlreadyDownloaded())
    }

    let _: Option<User> = data.db.update(("users", user.id.unwrap().id.to_string()))
        .patch(PatchOp::add("downloaded", Thing::from(map_id))).await.map_err(APIError::database_error)?;

    Ok("Ok!".to_string())
}

pub async fn remove(request: Request<Incoming>, identifier: UniqueIdentifier, data: &SiteData) -> Result<String, APIError> {
    let (map, user, _map_id) = get_map_request(request, identifier, data, SiteAction::Download).await?;

    let _: Option<User> = data.db.update(("users", user.id.unwrap().id.to_string()))
        .patch(PatchOp::remove(&*format!("downloaded/{}", user.downloaded.iter().position(|e| e == map.id.as_ref().unwrap())
            .ok_or(APIError::AlreadyDownloaded())?))).await.map_err(APIError::database_error)?;

    Ok("Ok!".to_string())
}